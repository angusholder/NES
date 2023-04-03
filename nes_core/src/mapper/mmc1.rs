use std::ops::Range;
use log::{trace};
use crate::cartridge::{NametableMirroring};
use crate::mapper;
use crate::mapper::memory_map::MemoryMap;
use crate::mapper::RawMapper;

/// Mapper 1: MMC1
/// https://www.nesdev.org/wiki/MMC1
pub struct MMC1Mapper {
    // See https://www.nesdev.org/wiki/MMC1#Registers
    prg_mode: PRGMode,
    chr_mode: CHRMode,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,

    shift_register: u8,
    shift_counter: u32,
}

#[derive(Debug)]
enum CHRMode {
    Switch8KiB,
    SwitchTwo4KiB,
}

#[derive(Debug)]
enum PRGMode {
    Switch32KiB,
    FixedFirstSwitchLast,
    FixedLastSwitchFirst,
}

impl MMC1Mapper {
    pub fn new() -> Self {
        MMC1Mapper {
            prg_mode: PRGMode::FixedLastSwitchFirst,
            chr_mode: CHRMode::Switch8KiB,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            shift_register: 0,
            shift_counter: 0,
        }
    }

    fn write_register(&mut self, memory: &mut MemoryMap, addr: u16, value: u8) {
        if value & 0x80 != 0 {
            trace!("Resetting state");
            self.reset(memory);
            return;
        }

        let mut new_sr = self.shift_register >> 1;
        if value & 1 != 0 {
            new_sr |= 0b1_0000;
        }
        self.shift_register = new_sr & 0b11111;
        self.shift_counter += 1;

        if self.shift_counter >= 5 {
            match addr & 0xF000 {
                0x8000 | 0x9000 => self.write_control_register(memory, self.shift_register),
                0xA000 | 0xB000 => {
                    self.chr_bank_0 = self.shift_register;
                    trace!("Set CHR bank 0 = {}", self.chr_bank_0);
                    self.sync_mappings(memory);
                }
                0xC000 | 0xD000 => {
                    self.chr_bank_1 = self.shift_register;
                    trace!("Set CHR bank 1 = {}", self.chr_bank_1);
                    self.sync_mappings(memory);
                }
                0xE000 | 0xF000 => {
                    self.prg_bank = self.shift_register;
                    trace!("Set PRG bank = {}", self.prg_bank);
                    self.sync_mappings(memory);
                }
                _ => unreachable!("{addr:04X}"),
            }
            self.reset_shift_register();
        }
    }

    fn reset(&mut self, memory: &mut MemoryMap) {
        self.reset_shift_register();
        // Initially set to PRGMode::FixedLastSwitchFirst
        self.prg_mode =PRGMode::FixedLastSwitchFirst;
        self.sync_mappings(memory);
    }

    fn reset_shift_register(&mut self) {
        self.shift_register = 0;
        self.shift_counter = 0;
    }

    fn write_control_register(&mut self, memory: &mut MemoryMap, byte: u8) {
        let mirroring = match byte & 0b11 {
            0 => NametableMirroring::SingleScreenLowerBank,
            1 => NametableMirroring::SingleScreenUpperBank,
            2 => NametableMirroring::Vertical,
            3 => NametableMirroring::Horizontal,
            _ => unreachable!(),
        };
        memory.set_nametable_mirroring(mirroring);
        self.prg_mode = match byte >> 2 & 0b11 {
            0 | 1 => PRGMode::Switch32KiB,
            2 => PRGMode::FixedFirstSwitchLast,
            3 => PRGMode::FixedLastSwitchFirst,
            _ => unreachable!(),
        };
        self.chr_mode = match byte >> 4 & 1 {
            0 => CHRMode::Switch8KiB,
            1 => CHRMode::SwitchTwo4KiB,
            _ => unreachable!(),
        };
        trace!("Set mapper control register to {byte:02X}:");
        trace!("> mirroring = {:?}", mirroring);
        trace!("> PRG mode = {:?}", self.prg_mode);
        trace!("> CHR mode = {:?}", self.chr_mode);
        self.sync_mappings(memory);
    }

    fn sync_mappings(&self, memory: &mut MemoryMap) {
        match self.prg_mode {
            PRGMode::Switch32KiB => {
                memory.map_prg_16k(0, (self.prg_bank & !1) as i32);
                memory.map_prg_16k(1, (self.prg_bank & !1) as i32 + 1);
            }
            PRGMode::FixedFirstSwitchLast => {
                memory.map_prg_16k(0, 0);
                memory.map_prg_16k(0, self.prg_bank as i32);
            }
            PRGMode::FixedLastSwitchFirst => {
                memory.map_prg_16k(0, self.prg_bank as i32);
                memory.map_prg_16k(0, -1);
            }
        }

        match self.chr_mode {
            CHRMode::Switch8KiB => {
                memory.map_chr_8k(self.chr_bank_0 >> 1);
            }
            CHRMode::SwitchTwo4KiB => {
                memory.map_chr_4k(0, self.chr_bank_0);
                memory.map_chr_4k(1, self.chr_bank_1);
            }
        }
    }
}

impl RawMapper for MMC1Mapper {
    fn init_memory_map(&self, memory: &mut MemoryMap) {
        memory.configure_chr_ram(8192);
        memory.set_nametable_mirroring(NametableMirroring::SingleScreenLowerBank);
        self.sync_mappings(memory);
    }

    fn write_main_bus(&mut self, memory: &mut MemoryMap, addr: u16, value: u8) {
        match addr {
            0x8000..=0xFFFF => {
                self.write_register(memory, addr, value);
            }
            _ => mapper::out_of_bounds_write("CPU memory map", addr, value)
        }
    }
}
