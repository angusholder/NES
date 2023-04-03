use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper;
use crate::mapper::{RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// https://www.nesdev.org/wiki/MMC2
/// Only used for Mike Tyson's Punch Out - https://nescartdb.com/profile/view/317/mike-tysons-punch-out
pub struct MMC2Mapper {
    map: MemoryMap,

    prg_bank: u8,
    chr_bank_0: [u8; 2],
    chr_bank_1: [u8; 2],
    chr_selector_0: BankSelector,
    chr_selector_1: BankSelector,
}

impl MMC2Mapper {
    pub fn new(cart: Cartridge) -> MMC2Mapper {
        let mut map = MemoryMap::new(&cart);
        map.set_nametable_mirroring(NametableMirroring::Horizontal);
        let mut mapper = MMC2Mapper {
            map,

            prg_bank: 0,
            chr_bank_0: [0, 0],
            chr_bank_1: [0, 0],
            chr_selector_0: BankSelector::FE,
            chr_selector_1: BankSelector::FE,
        };
        mapper.sync_mappings();
        mapper
    }

    fn sync_mappings(&mut self) {
        self.map.map_prg_8k(0, self.prg_bank as i32);
        self.map.map_prg_8k(1, -3);
        self.map.map_prg_8k(2, -2);
        self.map.map_prg_8k(3, -1);

        self.map.map_chr_4k(0, self.chr_bank_0[self.chr_selector_0 as usize]);
        self.map.map_chr_4k(1, self.chr_bank_1[self.chr_selector_1 as usize]);
    }
}

#[derive(Clone, Copy, Debug)]
enum BankSelector {
    FD = 0,
    FE = 1,
}

impl RawMapper for MMC2Mapper {
    fn write_main_bus(&mut self, addr: u16, value: u8) {
        let chr_bank_addr = value & 0b1_1111;
        match addr {
            0xA000..=0xAFFF => {
                self.prg_bank = value & 0xF;
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
                let mirroring = match value & 1 {
                    0 => NametableMirroring::Vertical,
                    1 => NametableMirroring::Horizontal,
                    _ => unreachable!(),
                };
                self.map.set_nametable_mirroring(mirroring);
            }
            _ => {
                mapper::out_of_bounds_write("cart", addr, value);
            }
        }
        self.sync_mappings();
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        self.map.read_main_bus(addr)
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        let result: u8 = self.map.access_ppu_bus(addr, value, write);

        // Update the selectors *after* performing the read/write
        match addr {
            0x0FD0..=0x0FDF => self.chr_selector_0 = BankSelector::FD,
            0x0FE0..=0x0FEF => self.chr_selector_0 = BankSelector::FE,
            0x1FD0..=0x1FDF => self.chr_selector_1 = BankSelector::FD,
            0x1FE0..=0x1FEF => self.chr_selector_1 = BankSelector::FE,
            _ => {
                // Skip sync_mappings()
                return result;
            }
        }

        self.sync_mappings();
        result
    }
}
