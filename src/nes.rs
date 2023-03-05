use std::fmt::{Display, Formatter};
use std::io::Write;
use crate::mapper::{Mapper};
use crate::{disassemble, input, cpu, ppu};
use crate::input::InputState;
use crate::ppu::PPU;

#[allow(non_snake_case)]
pub struct NES {
    remaining_cycles: i64,
    total_cycles: u64,

    pub A: u8,
    pub X: u8,
    pub Y: u8,
    /// An 'empty' stack that grows downwards, in memory area 0x0100 - 0x01FF.
    /// SP points to the next free location.
    /// See https://www.nesdev.org/wiki/Stack
    pub SP: u8,
    pub SR: StatusRegister,

    pub PC: u16,

    pub ram: [u8; 2048],
    pub ppu: PPU,

    pub mapper: Mapper,

    pub trigger_nmi: bool,
    trigger_irq: bool,

    pub trace_output: Option<Box<dyn Write>>,

    pub input: InputState,
}

pub const CYCLES_PER_FRAME: u64 = 29781;

/// https://www.nesdev.org/wiki/Status_flags
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub struct StatusRegister {
    /// Carry
    pub C: bool,
    /// Zero
    pub Z: bool,
    /// Interrupt disable
    pub I: bool,
    /// Decimal
    pub D: bool,
    /// Overflow
    pub V: bool,
    /// Negative
    pub N: bool,
}

impl Display for StatusRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_char(if self.C { 'C' } else { 'c' })?;
        f.write_char(if self.Z { 'Z' } else { 'z' })?;
        f.write_char(if self.I { 'I' } else { 'i' })?;
        f.write_char(if self.D { 'D' } else { 'd' })?;
        f.write_char(if self.V { 'V' } else { 'v' })?;
        f.write_char(if self.N { 'N' } else { 'n' })?;
        Ok(())
    }
}

impl StatusRegister {
    pub const FLAG_C: u8 = 0b00000001;
    pub const FLAG_Z: u8 = 0b00000010;
    pub const FLAG_I: u8 = 0b00000100;
    pub const FLAG_D: u8 = 0b00001000;
    pub const FLAG_B: u8 = 0b00010000;
    pub const FLAG_U: u8 = 0b00100000;
    pub const FLAG_V: u8 = 0b01000000;
    pub const FLAG_N: u8 = 0b10000000;
}

impl StatusRegister {
    pub fn to_byte(&self) -> u8 {
        return
            (self.C as u8     ) |
            ((self.Z as u8) << 1) |
            ((self.I as u8) << 2) |
            ((self.D as u8) << 3) |
            StatusRegister::FLAG_U |
            ((self.V as u8) << 6) |
            ((self.N as u8) << 7);
    }

    pub fn from_byte(value: u8) -> StatusRegister {
        return StatusRegister {
            C: value & StatusRegister::FLAG_C != 0,
            Z: value & StatusRegister::FLAG_Z != 0,
            I: value & StatusRegister::FLAG_I != 0,
            D: value & StatusRegister::FLAG_D != 0,
            V: value & StatusRegister::FLAG_V != 0,
            N: value & StatusRegister::FLAG_N != 0,
        };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Interrupt {
    RESET,
    NMI,
    IRQ,
    BRK,
}

impl Interrupt {
    pub fn get_vector_address(self) -> u16 {
        match self {
            Interrupt::RESET => 0xFFFC,
            Interrupt::NMI => 0xFFFA,
            Interrupt::IRQ => 0xFFFE,
            Interrupt::BRK => 0xFFFE,
        }
    }
}

impl NES {
    pub fn new(mapper: Mapper, trace_output: Option<Box<dyn Write>>) -> NES {
        NES {
            A: 0,
            X: 0,
            Y: 0,
            SP: 0xFD,
            PC: 0,
            SR: StatusRegister::from_byte(0),
            ram: [0; 0x800],
            remaining_cycles: 0,
            total_cycles: 0,
            ppu: PPU::new(mapper.clone()),
            mapper,
            trigger_nmi: false,
            trigger_irq: false,
            trace_output,

            input: InputState::new(),
        }
    }

    pub fn power_on(&mut self) {
        self.remaining_cycles = 0;

        self.SR = StatusRegister::from_byte(0);
        self.A = 0;
        self.X = 0;
        self.Y = 0;
        self.SP = 0;

        self.ram.fill(0xCC);

        self.interrupt(Interrupt::RESET);
    }

    pub fn simulate_frame(&mut self) {
        self.remaining_cycles += CYCLES_PER_FRAME as i64;
        while self.remaining_cycles > 0 {
            if self.trigger_nmi {
                self.interrupt(Interrupt::NMI);
                self.trigger_nmi = false;
            } else if self.trigger_irq && !self.SR.I {
                self.interrupt(Interrupt::IRQ);
            }
            if self.trace_output.is_some() {
                disassemble::disassemble(self);
            }
            cpu::emulate_instruction(self);
        }
    }

    pub fn interrupt(&mut self, interrupt: Interrupt) {
        if interrupt != Interrupt::RESET {
            self.push16(self.PC);
            let mut sr = self.SR.to_byte();
            if interrupt == Interrupt::BRK {
                sr |= StatusRegister::FLAG_B; // B flag set to indicate software IRQ
            }
            self.push8(sr);
        } else {
            self.SP = self.SP.wrapping_sub(3);
            self.tick(); self.tick(); self.tick();
        }

        self.SR.I = true;
        self.PC = self.read_addr(interrupt.get_vector_address());
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        self.tick();
        if addr < 0x2000 {
            return self.ram[addr as usize % 0x800];
        } else if addr < 0x4000 {
            return ppu::ppu_read_register(self, addr);
        } else if addr == input::JOYPAD_1 || addr == input::JOYPAD_2 {
            return self.input.handle_register_access(addr, 0, false);
        } else if addr < 0x4020 {
            println!("Unimplemented APU mem read at ${addr:04X}");
            return 0;
        } else {
            return self.mapper.read_main_bus(addr);
        }
    }

    pub fn read_addr(&mut self, addr: u16) -> u16 {
        let low = self.read8(addr);
        let high = self.read8(addr.wrapping_add(1));
        (high as u16) << 8 | (low as u16)
    }

    pub fn read_code(&mut self) -> u8 {
        let val = self.read8(self.PC);
        self.PC = self.PC.wrapping_add(1);
        val
    }

    pub fn read_code_addr(&mut self) -> u16 {
        let low = self.read_code();
        let high = self.read_code();
        (high as u16) << 8 | (low as u16)
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        self.tick();
        if addr < 0x2000 {
            self.ram[addr as usize % 0x800] = val;
        } else if addr < 0x4000 {
            ppu::ppu_write_register(self, addr, val);
        } else if addr == input::JOYPAD_1 || addr == input::JOYPAD_2 {
            self.input.handle_register_access(addr, val, true);
        } else if addr < 0x4020 {
            println!("Unimplemented APU mem write {addr:04X} of {val:02X}");
        } else {
            self.mapper.write_main_bus(addr, val);
        }
    }

    pub fn reset_state(&mut self) {
        self.SP = 0xFD;
    }

    pub fn set_status_register(&mut self, value: u8) {
        self.SR = StatusRegister::from_byte(value);
    }

    pub fn get_status_register(&mut self) -> u8 {
        self.SR.to_byte()
    }

    pub fn push8(&mut self, value: u8) {
        self.write8(0x0100 + self.SP as u16, value);
        self.SP = self.SP.wrapping_sub(1);
    }

    pub fn pop8(&mut self) -> u8 {
        self.SP = self.SP.wrapping_add(1);
        self.read8(0x0100 + self.SP as u16)
    }

    pub fn push16(&mut self, value: u16) {
        self.push8((value >> 8) as u8);
        self.push8((value & 0xFF) as u8);
    }

    pub fn pop16(&mut self) -> u16 {
        let low = self.pop8();
        let high = self.pop8();
        (high as u16) << 8 | (low as u16)
    }

    pub fn tick(&mut self) {
        self.remaining_cycles -= 1;
        self.total_cycles += 1;
        ppu::ppu_step(self);
        ppu::ppu_step(self);
        ppu::ppu_step(self);
    }

    pub fn get_cycles(&self) -> u64 {
        self.total_cycles
    }

    pub fn save_cycles(&self) -> CycleSavepoint {
        CycleSavepoint {
            remaining_cycles: self.remaining_cycles,
        }
    }

    pub fn restore_cycles(&mut self, savepoint: CycleSavepoint) {
        self.remaining_cycles = savepoint.remaining_cycles;
    }
}

pub struct CycleSavepoint {
    remaining_cycles: i64,
}
