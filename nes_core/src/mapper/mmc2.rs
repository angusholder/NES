use log::info;
use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper;
use crate::mapper::RawMapper;

/// https://www.nesdev.org/wiki/MMC2
/// Only used for Mike Tyson's Punch Out - https://nescartdb.com/profile/view/317/mike-tysons-punch-out
pub struct MMC2Mapper {
    prg_rom: Box<[u8; 128 * 1024]>,
    chr_rom: Box<[u8; 128 * 1024]>,

    prg_bank: usize,
    chr_bank_0: [usize; 2],
    chr_bank_1: [usize; 2],
    chr_selector_0: BankSelector,
    chr_selector_1: BankSelector,

    mirroring: NametableMirroring,
    nametables: [u8; 0x800],
}

impl MMC2Mapper {
    pub fn new(cart: Cartridge) -> MMC2Mapper {
        MMC2Mapper {
            prg_rom: cart.prg_rom.try_into().unwrap(),
            chr_rom: cart.chr_rom.try_into().unwrap(),

            prg_bank: 0,
            chr_bank_0: [0, 0],
            chr_bank_1: [0, 0],
            chr_selector_0: BankSelector::FE,
            chr_selector_1: BankSelector::FE,

            mirroring: NametableMirroring::Horizontal,
            nametables: [0; 0x800],
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let chr_bank_addr = (value & 0b1_1111) as usize * 4 * 1024;
        match addr {
            0xA000..=0xAFFF => {
                self.prg_bank = (value & 0xF) as usize * 8 * 1024;
            }
            0xB000..=0xBFFF => {
                self.chr_bank_0[BankSelector::FD as usize] = chr_bank_addr;
            }
            0xC000..=0xCFFF => {
                self.chr_bank_0[BankSelector::FE as usize] = chr_bank_addr;
            }
            0xD000..=0xDFFF => {
                self.chr_bank_1[BankSelector::FD as usize] = chr_bank_addr;
            }
            0xE000..=0xEFFF => {
                self.chr_bank_1[BankSelector::FE as usize] = chr_bank_addr;
            }
            0xF000..=0xFFFF => {
                self.mirroring = match value & 1 {
                    0 => NametableMirroring::Vertical,
                    1 => NametableMirroring::Horizontal,
                    _ => unreachable!(),
                }
            }
            _ => {
                mapper::out_of_bounds_write("cart", addr, value);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum BankSelector {
    FD = 0,
    FE = 1,
}

impl RawMapper for MMC2Mapper {
    fn access_main_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        if write {
            self.write_register(addr, value);
            return 0;
        }

        match addr {
            0x8000..=0x9FFF => {
                self.prg_rom[self.prg_bank + (addr & 0x1FFF) as usize]
            }
            0xA000..=0xFFFF => {
                let offset = (addr - 0xA000) as usize;
                let base = self.prg_rom.len() - 3 * 8 * 1024;
                self.prg_rom[base + offset]
            }
            _ => {
                mapper::out_of_bounds_read("cart", addr)
            }
        }
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x0FFF if !write => {
                let base = self.chr_bank_0[self.chr_selector_0 as usize];
                match addr {
                    0x0FD0..=0x0FDF => self.chr_selector_0 = BankSelector::FD,
                    0x0FE0..=0x0FEF => self.chr_selector_0 = BankSelector::FE,
                    _ => {}
                }
                self.chr_rom[base + (addr as usize & 0x0FFF)]
            }
            0x1000..=0x1FFF if !write => {
                let base = self.chr_bank_1[self.chr_selector_1 as usize];
                match addr {
                    0x1FD0..=0x1FDF => self.chr_selector_1 = BankSelector::FD,
                    0x1FE0..=0x1FEF => self.chr_selector_1 = BankSelector::FE,
                    _ => {}
                }
                self.chr_rom[base + (addr as usize & 0x0FFF)]
            }
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                mapper::access_nametable(&mut self.nametables, self.mirroring, addr & 0x2FFF, value, write)
            }
            _ => {
                mapper::out_of_bounds_access("PPU memory space", addr, value, write)
            }
        }
    }
}
