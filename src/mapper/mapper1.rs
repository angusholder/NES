use std::ops::Range;
use log::warn;
use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper::mapper0::access_nametable;
use crate::mapper::RawMapper;

/// https://www.nesdev.org/wiki/MMC1
pub struct MMC1Mapper {
    rom: Vec<u8>,

    // See https://www.nesdev.org/wiki/MMC1#Registers
    prg_mode: PRGMode,
    chr_mode: CHRMode,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,

    shift_register: u8,

    mirroring: MMC1Mirroring,
    nametables: [u8; 0x800],
}

enum CHRMode {
    Switch8KiB,
    SwitchTwo4KiB,
}

enum PRGMode {
    Switch32KiB,
    FixedFirstSwitchLast,
    FixedLastSwitchFirst,
}

#[derive(Debug)]
enum MMC1Mirroring {
    OneScreenLowerBank = 0,
    OneScreenUpperBank = 1,
    Vertical = 2,
    Horizontal = 3,
}

impl MMC1Mapper {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            rom: cart.prg_rom,
            prg_mode: PRGMode::FixedLastSwitchFirst,
            chr_mode: CHRMode::Switch8KiB,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            shift_register: 0,
            mirroring: MMC1Mirroring::OneScreenLowerBank,
            nametables: [0; 0x800],
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if value & 0x80 != 0 {
            self.reset();
            return;
        }

        let is_done = self.shift_register & 1 != 0;
        let mut new_sr = self.shift_register >> 1;
        if value & 1 != 0 {
            new_sr |= 0b1_0000;
        }
        self.shift_register = new_sr;

        if is_done {
            match addr & 0xE000 {
                0x8000 => self.write_control_register(self.shift_register),
                0xA000 => self.chr_bank_0 = self.shift_register,
                0xC000 => self.chr_bank_1 = self.shift_register,
                0xE000 => self.prg_bank = self.shift_register,
                _ => unreachable!(),
            }
            self.reset_shift_register();
        }
    }

    fn reset(&mut self) {
        self.reset_shift_register();
        // Initially set to PRGMode::FixedLastSwitchFirst
        self.write_control_register(0xC);
    }

    fn reset_shift_register(&mut self) {
        self.shift_register = 0b10000;
    }

    fn write_control_register(&mut self, byte: u8) {
        self.mirroring = match byte & 0b11 {
            0 => MMC1Mirroring::OneScreenLowerBank,
            1 => MMC1Mirroring::OneScreenUpperBank,
            2 => MMC1Mirroring::Vertical,
            3 => MMC1Mirroring::Horizontal,
            _ => unreachable!(),
        };
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
    }
}

impl RawMapper for MMC1Mapper {
    fn access_main_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        if write {
            self.write_register(addr, value);
        }

        let low_bank: Range<usize>;
        let high_bank: Range<usize>;
        match self.prg_mode {
            PRGMode::Switch32KiB => {
                let base_addr = (self.prg_bank & !1) as usize * 16*1024;
                low_bank = base_addr..base_addr + 16*1024;
                high_bank = base_addr + 16*1024..base_addr + 32*1024;
            }
            PRGMode::FixedFirstSwitchLast => {
                low_bank = 0..16*1024;
                let base_addr = self.prg_bank as usize * 16*1024;
                high_bank = base_addr..base_addr + 16*1024;
            }
            PRGMode::FixedLastSwitchFirst => {
                let base_addr = self.prg_bank as usize * 16*1024;
                low_bank = base_addr..base_addr + 16*1024;
                high_bank = self.rom.len() - 16*1024..self.rom.len();
            }
        }

        if addr < 0xC000 {
            self.rom[low_bank][addr as usize - 0x8000]
        } else {
            self.rom[high_bank][addr as usize - 0xC000]
        }
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                if write {
                    warn!("Attempted to write to CHR ROM: {addr:04X} = {value:02X}");
                }
                match self.chr_mode {
                    CHRMode::Switch8KiB => {
                        let base_addr = (self.chr_bank_0 & !1) as usize  * 4 * 1024;
                        self.rom[base_addr + addr as usize]
                    }
                    CHRMode::SwitchTwo4KiB => {
                        let base_addr = if addr < 0x1000 {
                            self.chr_bank_0 as usize * 4 * 1024
                        } else {
                            self.chr_bank_1 as usize * 4 * 1024
                        };
                        self.rom[base_addr + addr as usize]
                    }
                }
            },
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                let nt_mirroring = match self.mirroring {
                    MMC1Mirroring::Horizontal => NametableMirroring::Horizontal,
                    MMC1Mirroring::Vertical => NametableMirroring::Vertical,
                    _ => {
                        warn!("Unimplemented mirroring mode {:?}, defaulting to horizontal", self.mirroring);
                        NametableMirroring::Horizontal
                    }
                };
                let ptr = access_nametable(&mut self.nametables, nt_mirroring, addr & 0x2FFF);
                if write {
                    *ptr = value;
                }
                *ptr
            }
            _ => {
                panic!("Attempted to access CHR outside of range: {addr:04X}")
            }
        }

    }
}
