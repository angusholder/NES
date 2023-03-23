use std::ops::Range;
use log::{trace, warn};
use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper;
use crate::mapper::access_nametable;
use crate::mapper::RawMapper;

/// Mapper 1: MMC1
/// https://www.nesdev.org/wiki/MMC1
pub struct MMC1Mapper {
    prg_rom: Box<[u8]>,
    chr_ram: [u8; 8192],
    wram: Option<[u8; 8192]>,

    // See https://www.nesdev.org/wiki/MMC1#Registers
    prg_mode: PRGMode,
    chr_mode: CHRMode,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,

    shift_register: u8,
    shift_counter: u32,

    mirroring: NametableMirroring,
    nametables: [u8; 0x800],
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
    pub fn new(cart: Cartridge) -> Self {
        if cart.prg_ram_battery_backed {
            warn!("Battery-backed PRG RAM not supported");
        }

        Self {
            prg_rom: cart.prg_rom.into_boxed_slice(),
            chr_ram: [0; 8192],
            wram: if cart.prg_ram_size == 8*1024 { Some([0; 8*1024]) } else { None },
            prg_mode: PRGMode::FixedLastSwitchFirst,
            chr_mode: CHRMode::Switch8KiB,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            shift_register: 0,
            shift_counter: 0,
            mirroring: NametableMirroring::SingleScreenLowerBank,
            nametables: [0; 0x800],
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if value & 0x80 != 0 {
            trace!("Resetting state");
            self.reset();
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
                0x8000 | 0x9000 => self.write_control_register(self.shift_register),
                0xA000 | 0xB000 => {
                    self.chr_bank_0 = self.shift_register;
                    trace!("Set CHR bank 0 = {}", self.chr_bank_0);
                }
                0xC000 | 0xD000 => {
                    self.chr_bank_1 = self.shift_register;
                    trace!("Set CHR bank 1 = {}", self.chr_bank_1);
                }
                0xE000 | 0xF000 => {
                    self.prg_bank = self.shift_register;
                    trace!("Set PRG bank = {}", self.prg_bank);
                }
                _ => unreachable!("{addr:04X}"),
            }
            self.reset_shift_register();
        }
    }

    fn reset(&mut self) {
        self.reset_shift_register();
        // Initially set to PRGMode::FixedLastSwitchFirst
        self.prg_mode =PRGMode::FixedLastSwitchFirst;
    }

    fn reset_shift_register(&mut self) {
        self.shift_register = 0;
        self.shift_counter = 0;
    }

    fn write_control_register(&mut self, byte: u8) {
        self.mirroring = match byte & 0b11 {
            0 => NametableMirroring::SingleScreenLowerBank,
            1 => NametableMirroring::SingleScreenUpperBank,
            2 => NametableMirroring::Vertical,
            3 => NametableMirroring::Horizontal,
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
        trace!("Set mapper control register to {byte:02X}:");
        trace!("> mirroring = {:?}", self.mirroring);
        trace!("> PRG mode = {:?}", self.prg_mode);
        trace!("> CHR mode = {:?}", self.chr_mode);
    }
}

const WRAM_RANGE: Range<u16> = 0x6000..0x8000;

impl RawMapper for MMC1Mapper {
    fn write_main_bus(&mut self, addr: u16, value: u8) {
        if WRAM_RANGE.contains(&addr) {
            if let Some(wram) = self.wram.as_mut() {
                wram[addr as usize & 0x1FFF] = value;
                return;
            }
        }

        self.write_register(addr, value);
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        const BANK_SIZE: usize = 16 * 1024;

        if WRAM_RANGE.contains(&addr) {
            if let Some(wram) = self.wram.as_ref() {
                return wram[addr as usize & 0x1FFF];
            }
        }

        let low_bank: usize;
        let high_bank: usize;
        match self.prg_mode {
            PRGMode::Switch32KiB => {
                let base_addr = (self.prg_bank & !1) as usize * BANK_SIZE;
                low_bank = base_addr;
                high_bank = base_addr + BANK_SIZE;
            }
            PRGMode::FixedFirstSwitchLast => {
                low_bank = 0;
                let base_addr = self.prg_bank as usize * BANK_SIZE;
                high_bank = base_addr;
            }
            PRGMode::FixedLastSwitchFirst => {
                let base_addr = self.prg_bank as usize * BANK_SIZE;
                low_bank = base_addr;
                high_bank = self.prg_rom.len() - BANK_SIZE;
            }
        }

        if addr >= 0x8000 && addr < 0xC000 {
            self.prg_rom[low_bank..low_bank+BANK_SIZE][addr as usize - 0x8000]
        } else if addr >= 0xC000 {
            self.prg_rom[high_bank..high_bank+BANK_SIZE][addr as usize - 0xC000]
        } else {
            mapper::out_of_bounds_read("cartridge", addr)
        }
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let ptr = match self.chr_mode {
                    CHRMode::Switch8KiB => {
                        let base_addr = ((self.chr_bank_0 >> 1) as usize * 8 * 1024) % self.chr_ram.len();
                        &mut self.chr_ram[base_addr + addr as usize]
                    }
                    CHRMode::SwitchTwo4KiB => {
                        let base_addr = if addr < 0x1000 {
                            self.chr_bank_0 as usize * 4 * 1024
                        } else {
                            self.chr_bank_1 as usize * 4 * 1024
                        } % self.chr_ram.len();
                        &mut self.chr_ram[base_addr + (addr&0x0FFF) as usize]
                    }
                };
                if write {
                    *ptr = value;
                }
                *ptr
            },
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                access_nametable(&mut self.nametables, self.mirroring, addr & 0x2FFF, value, write)
            }
            _ => {
                mapper::out_of_bounds_access("CHR", addr, value, write)
            }
        }

    }
}
