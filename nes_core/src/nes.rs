use std::cell::Cell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use bitflags::bitflags;
use log::Level::Trace;
use serde::{Deserialize, Serialize};
use crate::mapper::{Mapper};
use crate::{cpu, ppu};
use crate::apu::{APU, APUSnapshot};
use crate::cartridge::Cartridge;
use crate::input::InputState;
use crate::ppu::{PPU, PPUSnapshot};

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

#[derive(Serialize, Deserialize)]
pub struct NESSnapshot {
    pub cpu: CPUSnapshot,
    pub input: InputState,
    pub ppu: PPUSnapshot,
    pub apu: APUSnapshot,
    // signals
    // mapper
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct CPUSnapshot {
    pub target_cycles: u64,
    pub total_cycles: u64,

    pub A: u8,
    pub X: u8,
    pub Y: u8,
    /// An 'empty' stack that grows downwards, in memory area 0x0100 - 0x01FF.
    /// SP points to the next free location.
    /// See https://www.nesdev.org/wiki/Stack
    pub SP: u8,
    pub SR: StatusRegister,

    pub PC: u16,

    pub ram: Box<[u8]>, // 2048 elements (serde doesn't support fixed-size arrays over 16 elems)
}

bitflags! {
    /// The CPU IRQ will continually trigger until the IRQ handler acknowledges all active IRQ sources.
    /// See https://www.nesdev.org/wiki/IRQ
    pub struct InterruptSource : u32 {
        const APU_DMC = 0x01;
        const APU_FRAME_COUNTER = 0x02;
        const MMC3 = 0x04;
        const VBLANK_NMI = 0x08;
    }
}

/// Holds all the IRQ/NMI signals for the whole console.
/// Any subsystem can raise an IRQ, and it will continue until the IRQ handler acknowledges
/// that specific subsystem's IRQ signal.
/// See https://www.nesdev.org/wiki/IRQ
pub struct Signals {
    signal: Cell<InterruptSource>,
}

impl Signals {
    pub fn new() -> Rc<Signals> {
        Rc::new(Signals {
            signal: Cell::new(InterruptSource::empty()),
        })
    }

    /// Is any interrupt signal active
    pub fn is_any_active(&self) -> bool {
        !self.signal.get().is_empty()
    }

    /// A subsystem is requesting interrupt
    pub fn request_interrupt(&self, source: InterruptSource) {
        self.signal.set(self.signal.get() | source);
    }

    /// A subsystem's interrupt signal has been acknowledged, and can be dismissed now.
    pub fn acknowledge_interrupt(&self, source: InterruptSource) {
        self.signal.set(self.signal.get() - source);
    }

    /// Is a specific interrupt source active
    pub fn is_active(&self, source: InterruptSource) -> bool {
        self.signal.get().contains(source)
    }
}

pub const CYCLES_PER_FRAME: u64 = 29829;

/// https://www.nesdev.org/wiki/Status_flags
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
    /// The B status flag doesn't physically exist inside the CPU, and only appears as different values being pushed for bit 4 of the saved status bits by PHP, BRK, and NMI/IRQ.
    /// https://www.nesdev.org/wiki/CPU_interrupts
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
            V: value & StatusRegister::FLAG_V != 0,
            N: value & StatusRegister::FLAG_N != 0,
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
            ppu: PPU::new(mapper.clone(), signals.clone()),

            input: InputState::new(),
            apu: APU::new(mapper.clone(), signals.clone()),
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

        self.do_reset_interrupt();

        // Power-up and reset have the effect of writing $00, silencing all channels.
        self.apu.write_status_register(0x00);
    }

    pub fn simulate_frame(&mut self) {
        self.target_cycles += CYCLES_PER_FRAME;
        while self.target_cycles > self.total_cycles {
            if self.signals.is_any_active() {
                self.handle_interrupt();
            }
            cpu::emulate_instruction(self);
        }
    }

    fn handle_interrupt(&mut self) {
        if self.signals.is_active(InterruptSource::VBLANK_NMI) {
            self.signals.acknowledge_interrupt(InterruptSource::VBLANK_NMI);
            self.do_nmi_interrupt();
        } else if self.signals.is_any_active() && !self.SR.I {
            self.do_irq_interrupt();
        }
    }

    pub fn do_reset_interrupt(&mut self) {
        // https://www.nesdev.org/wiki/CPU_interrupts
        self.read8(self.PC); // fetch opcode (and discard it)
        self.read8(self.PC); // read next instruction byte (actually the same as above, since PC increment is suppressed. Also discarded.)

        // The push of PC and SP is suppressed for RESET, but the cycles and SP decrement still occur.
        self.SP = self.SP.wrapping_sub(3);
        self.tick(); self.tick(); self.tick();

        self.SR.I = true;
        self.PC = self.read_addr(0xFFFC);
    }

    pub fn do_nmi_interrupt(&mut self) {
        // https://www.nesdev.org/wiki/CPU_interrupts
        self.read8(self.PC); // fetch opcode (and discard it)
        self.read8(self.PC); // read next instruction byte (actually the same as above, since PC increment is suppressed. Also discarded.)

        self.push16(self.PC);
        self.push8(self.SR.to_byte());
        self.SR.I = true;
        self.PC = self.read_addr(0xFFFA);
    }

    pub fn do_brk_interrupt(&mut self) {
        // https://www.nesdev.org/wiki/CPU_interrupts
        self.read8(self.PC); // read next instruction byte (actually the same as above, since PC increment is suppressed. Also discarded.)

        self.push16(self.PC);
        self.push8(self.SR.to_byte() | StatusRegister::FLAG_B);
        self.SR.I = true;
        self.PC = self.read_addr(0xFFFE);
    }

    pub fn do_irq_interrupt(&mut self) {
        // https://www.nesdev.org/wiki/CPU_interrupts
        self.read8(self.PC); // fetch opcode (and discard it)
        self.read8(self.PC); // read next instruction byte (actually the same as above, since PC increment is suppressed. Also discarded.)

        self.push16(self.PC);
        self.push8(self.SR.to_byte());
        self.SR.I = true;
        self.PC = self.read_addr(0xFFFE);
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        self.tick();
        self.read8_no_tick(addr)
    }

    pub fn read8_no_tick(&mut self, addr: u16) -> u8 {
        match addr {
            0x4020..=0xFFFF =>
                self.mapper.read_main_bus(addr),
            0x0000..=0x1FFF =>
                self.ram[addr as usize % 0x800],
            0x2000..=0x3FFF =>
                self.ppu.read_register(addr),
            0x4016 =>
                self.input.read_joypad_1(),
            0x4017 =>
                self.input.read_joypad_2(),
            0x4000..=0x401F =>
                self.apu.read_register(addr),
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
                self.apu.write_register(addr, val),
        }
    }

    pub fn set_status_register(&mut self, value: u8) {
        self.SR = StatusRegister::from_byte(value);
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

        // 3 PPU cycles per CPU cycle
        self.ppu.step_cycle();
        self.ppu.step_cycle();
        self.ppu.step_cycle();

        // Most APU components run at half the CPU clock rate, but one needs the full clock rate
        self.apu.step_cycle(self.total_cycles);
    }

    pub fn get_cycles(&self) -> u64 {
        self.total_cycles
    }
}
