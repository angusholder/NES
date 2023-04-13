use std::cell::Cell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use bitflags::bitflags;
use log::Level::Trace;
use crate::mapper::{Mapper};
use crate::{cpu, ppu};
use crate::apu::APU;
use crate::cartridge::Cartridge;
use crate::input::InputState;
use crate::ppu::PPU;

#[allow(non_snake_case)]
pub struct NES {
    target_cycles: u64,
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

    pub trace_instructions: bool,

    pub ram: [u8; 2048],
    pub ppu: PPU,

    pub mapper: Mapper,

    pub input: InputState,
    pub apu: APU,
    signals: Rc<Signals>,
}

bitflags! {
    /// The CPU IRQ will continually trigger until the IRQ handler acknowledges all active IRQ sources.
    /// See https://www.nesdev.org/wiki/IRQ
    pub struct IRQSource : u32 {
        const APU_DMC = 0x01;
        const APU_FRAME_COUNTER = 0x02;
        const MMC3 = 0x04;
    }
}

/// Holds all the IRQ signals for the whole console.
/// Any subsystem can raise an IRQ, and it will continue until the IRQ handler acknowledges
/// that specific subsystem's IRQ signal.
/// See https://www.nesdev.org/wiki/IRQ
pub struct Signals {
    signal: Cell<IRQSource>,
}

impl Signals {
    pub fn new() -> Rc<Signals> {
        Rc::new(Signals {
            signal: Cell::new(IRQSource::empty()),
        })
    }

    /// Is any IRQ signal active
    pub fn is_any_active(&self) -> bool {
        !self.signal.get().is_empty()
    }

    /// A subsystem is requesting IRQ
    pub fn request_irq(&self, source: IRQSource) {
        self.signal.set(self.signal.get() | source);
    }

    /// A subsystem's IRQ signal has been acknowledged, and can be dismissed now.
    pub fn acknowledge_irq(&self, source: IRQSource) {
        self.signal.set(self.signal.get() - source);
    }

    /// Is a specific IRQ source active
    pub fn is_active(&self, source: IRQSource) -> bool {
        self.signal.get().contains(source)
    }
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
    /// BRK
    pub B: bool,
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
        f.write_char(if self.B { 'B' } else { 'b' })?;
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
        (self.C as u8       ) |
        ((self.Z as u8) << 1) |
        ((self.I as u8) << 2) |
        ((self.D as u8) << 3) |
        ((self.B as u8) << 4) |
        StatusRegister::FLAG_U |
        ((self.V as u8) << 6) |
        ((self.N as u8) << 7)
    }

    pub fn from_byte(value: u8) -> StatusRegister {
        StatusRegister {
            C: value & StatusRegister::FLAG_C != 0,
            Z: value & StatusRegister::FLAG_Z != 0,
            I: value & StatusRegister::FLAG_I != 0,
            D: value & StatusRegister::FLAG_D != 0,
            B: value & StatusRegister::FLAG_B != 0,
            V: value & StatusRegister::FLAG_V != 0,
            N: value & StatusRegister::FLAG_N != 0,
        }
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
    fn new(mapper: Mapper, signals: Rc<Signals>) -> NES {
        NES {
            A: 0,
            X: 0,
            Y: 0,
            SP: 0xFD,
            PC: 0,
            SR: StatusRegister::from_byte(0),
            ram: [0; 0x800],
            target_cycles: 0,
            total_cycles: 0,
            trace_instructions: log::log_enabled!(Trace),
            ppu: PPU::new(mapper.clone()),

            input: InputState::new(),
            apu: APU::new(signals.clone()),
            signals,
            mapper,
        }
    }

    pub fn from_cart(cart: Cartridge) -> NES {
        let signals = Signals::new();
        let mapper = Mapper::new(cart, signals.clone());
        NES::new(mapper, signals)
    }

    pub fn power_on(&mut self) {
        self.target_cycles = self.total_cycles;

        self.SR = StatusRegister::from_byte(0);
        self.A = 0;
        self.X = 0;
        self.Y = 0;
        self.SP = 0;

        self.ram.fill(0xCC);

        self.interrupt(Interrupt::RESET);

        // Power-up and reset have the effect of writing $00, silencing all channels.
        self.apu.write_status_register(0x00);
    }

    pub fn simulate_frame(&mut self) {
        self.target_cycles += CYCLES_PER_FRAME;
        while self.target_cycles > self.total_cycles {
            if self.ppu.request_nmi {
                self.interrupt(Interrupt::NMI);
                self.ppu.request_nmi = false;
            } else if self.signals.is_any_active() && !self.SR.I {
                self.interrupt(Interrupt::IRQ);
            }
            cpu::emulate_instruction(self);
        }
        // At the end of the frame, flush any remaining audio samples.
        self.apu.run_until_cycle(self.total_cycles);
    }

    pub fn interrupt(&mut self, interrupt: Interrupt) {
        if interrupt != Interrupt::RESET {
            self.push16(self.PC);
            self.push8(self.SR.to_byte());
            if interrupt == Interrupt::BRK {
                self.SR.B = true;
                self.tick();
            }
        } else {
            self.SP = self.SP.wrapping_sub(3);
            self.tick(); self.tick(); self.tick();
        }

        if interrupt != Interrupt::NMI {
            self.SR.I = true;
        }
        self.PC = self.read_addr(interrupt.get_vector_address());
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        self.tick();
        self.read8_no_tick(addr)
    }

    pub fn read8_no_tick(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF =>
                self.ram[addr as usize % 0x800],
            0x4020..=0xFFFF =>
                self.mapper.read_main_bus(addr),
            0x2000..=0x3FFF =>
                self.ppu.read_register(addr),
            0x4016 =>
                self.input.read_joypad_1(),
            0x4017 =>
                self.input.read_joypad_2(),
            0x4000..=0x401F =>
                self.apu.read_register(addr, self.total_cycles),
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
        match addr {
            0x0000..=0x1FFF =>
                self.ram[addr as usize % 0x800] = val,
            0x4020..=0xFFFF =>
                self.mapper.write_main_bus(addr, val),
            0x2000..=0x3FFF =>
                self.ppu.write_register(addr, val),
            0x4016 =>
                self.input.write_joypad_strobe(val),
            0x4014 =>
                ppu::do_oam_dma(self, val),
            0x4000..=0x401F =>
                self.apu.write_register(addr, val, self.total_cycles),
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
        self.total_cycles += 1;
        ppu::ppu_step(&mut self.ppu);
        ppu::ppu_step(&mut self.ppu);
        ppu::ppu_step(&mut self.ppu);
        self.apu.step_cycle(self.total_cycles);
    }

    pub fn get_cycles(&self) -> u64 {
        self.total_cycles
    }
}
