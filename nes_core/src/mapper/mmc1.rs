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

    // Calculated mappings
    prg_low_bank: usize,
    prg_high_bank: usize,
    chr_base_addrs: [usize; 2],
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

        let mut mapper = Self {
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

            prg_low_bank: 0,
            prg_high_bank: 0,
            chr_base_addrs: [0, 0],
        };
        mapper.sync_mappings();
        mapper
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
                    self.sync_mappings();
                }
                0xC000 | 0xD000 => {
                    self.chr_bank_1 = self.shift_register;
                    trace!("Set CHR bank 1 = {}", self.chr_bank_1);
                    self.sync_mappings();
                }
                0xE000 | 0xF000 => {
                    self.prg_bank = self.shift_register;
                    trace!("Set PRG bank = {}", self.prg_bank);
                    self.sync_mappings();
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
        self.sync_mappings()
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
        self.sync_mappings();
    }

    fn sync_mappings(&mut self) {
        const PRG_BANK_SIZE: usize = 16 * 1024;
        match self.prg_mode {
            PRGMode::Switch32KiB => {
                let base_addr = (self.prg_bank & !1) as usize * PRG_BANK_SIZE;
                self.prg_low_bank = base_addr;
                self.prg_high_bank = base_addr + PRG_BANK_SIZE;
            }
            PRGMode::FixedFirstSwitchLast => {
                self.prg_low_bank = 0;
                let base_addr = self.prg_bank as usize * PRG_BANK_SIZE;
                self.prg_high_bank = base_addr;
            }
            PRGMode::FixedLastSwitchFirst => {
                let base_addr = self.prg_bank as usize * PRG_BANK_SIZE;
                self.prg_low_bank = base_addr;
                self.prg_high_bank = self.prg_rom.len() - PRG_BANK_SIZE;
            }
        }

        match self.chr_mode {
            CHRMode::Switch8KiB => {
                let base_addr = ((self.chr_bank_0 >> 1) as usize * 8 * 1024) % self.chr_ram.len();
                self.chr_base_addrs[0] = base_addr;
                self.chr_base_addrs[1] = base_addr + 0x1000;
            }
            CHRMode::SwitchTwo4KiB => {
                self.chr_base_addrs[0] = (self.chr_bank_0 as usize * 4 * 1024) % self.chr_ram.len();
                self.chr_base_addrs[1] = (self.chr_bank_1 as usize * 4 * 1024) % self.chr_ram.len();
            }
        }
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
        match addr {
            0x6000..=0x7FFF => {
                if let Some(wram) = self.wram.as_ref() {
                    wram[addr as usize & 0x1FFF]
                } else {
                    mapper::out_of_bounds_read("WRAM", addr)
                }
            }
            0x8000..=0xBFFF => {
                self.prg_rom[self.prg_low_bank + (addr as usize & 0x3FFF)]
            }
            0xC000..=0xFFFF => {
                self.prg_rom[self.prg_high_bank + (addr as usize & 0x3FFF)]
            }
            _ => mapper::out_of_bounds_read("cartridge", addr),
        }
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x0FFF => {
                let ptr = &mut self.chr_ram[self.chr_base_addrs[0] + (addr&0x0FFF) as usize];
                if write {
                    *ptr = value;
                }
                *ptr
            },
            0x1000..=0x1FFF => {
                let ptr = &mut self.chr_ram[self.chr_base_addrs[1] + (addr&0x0FFF) as usize];
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
