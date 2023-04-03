use std::cell::{Cell};
use std::rc::Rc;
use crate::cartridge::{NametableMirroring};
use crate::mapper;
use crate::mapper::{PPUReadHook, RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// https://www.nesdev.org/wiki/MMC2
/// Only used for Mike Tyson's Punch Out - https://nescartdb.com/profile/view/317/mike-tysons-punch-out
pub struct MMC2Mapper {
    inner: Rc<Inner>,
}

struct Inner {
    prg_bank: Cell<u8>,
    chr_bank_0: [Cell<u8>; 2],
    chr_bank_1: [Cell<u8>; 2],
    chr_selector_0: Cell<BankSelector>,
    chr_selector_1: Cell<BankSelector>,
}

impl MMC2Mapper {
    pub fn new() -> MMC2Mapper {
        MMC2Mapper {
            inner: Rc::new(Inner {
                prg_bank: Cell::new(0),
                chr_bank_0: [Cell::new(0), Cell::new(0)],
                chr_bank_1: [Cell::new(0), Cell::new(0)],
                chr_selector_0: Cell::new(BankSelector::FE),
                chr_selector_1: Cell::new(BankSelector::FE),
            }),
        }
    }
}

impl Inner {
    fn sync_mappings(&self, memory: &mut MemoryMap) {
        memory.map_prg_8k(0, self.prg_bank.get() as i32);
        memory.map_prg_8k(1, -3);
        memory.map_prg_8k(2, -2);
        memory.map_prg_8k(3, -1);

        memory.map_chr_4k(0, self.chr_bank_0[self.chr_selector_0.get() as usize].get());
        memory.map_chr_4k(1, self.chr_bank_1[self.chr_selector_1.get() as usize].get());
    }
}

#[derive(Clone, Copy, Debug)]
enum BankSelector {
    FD = 0,
    FE = 1,
}

impl RawMapper for MMC2Mapper {
    fn init(&mut self, memory: &mut MemoryMap) {
        memory.set_nametable_mirroring(NametableMirroring::Horizontal);
        self.inner.sync_mappings(memory);
    }

    fn get_ppu_read_hook(&self) -> Option<Rc<PPUReadHook>> {
        let inner: Rc<Inner> = self.inner.clone();
        Some(Rc::new(move |memory: &mut MemoryMap, addr: u16| -> u8 {
            let result: u8 = memory.read_ppu_bus(addr);

            // Update the selectors *after* performing the read/write
            match addr {
                0x0FD0..=0x0FDF => {
                    inner.chr_selector_0.set(BankSelector::FD);
                    inner.sync_mappings(memory);
                }
                0x0FE0..=0x0FEF => {
                    inner.chr_selector_0.set(BankSelector::FE);
                    inner.sync_mappings(memory);
                }
                0x1FD0..=0x1FDF => {
                    inner.chr_selector_1.set(BankSelector::FD);
                    inner.sync_mappings(memory);
                }
                0x1FE0..=0x1FEF => {
                    inner.chr_selector_1.set(BankSelector::FE);
                    inner.sync_mappings(memory);
                }
                _ => {}
            }

            result
        }))
    }

    fn write_main_bus(&mut self, memory: &mut MemoryMap, addr: u16, value: u8) {
        let chr_bank_addr = value & 0b1_1111;
        match addr {
            0xA000..=0xAFFF => {
                self.inner.prg_bank.set(value & 0xF);
            }
            0xB000..=0xBFFF => {
                self.inner.chr_bank_0[BankSelector::FD as usize].set(chr_bank_addr);
            }
            0xC000..=0xCFFF => {
                self.inner.chr_bank_0[BankSelector::FE as usize].set(chr_bank_addr);
            }
            0xD000..=0xDFFF => {
                self.inner.chr_bank_1[BankSelector::FD as usize].set(chr_bank_addr);
            }
            0xE000..=0xEFFF => {
                self.inner.chr_bank_1[BankSelector::FE as usize].set(chr_bank_addr);
            }
            0xF000..=0xFFFF => {
                let mirroring = match value & 1 {
                    0 => NametableMirroring::Vertical,
                    1 => NametableMirroring::Horizontal,
                    _ => unreachable!(),
                };
                memory.set_nametable_mirroring(mirroring);
            }
            _ => {
                mapper::out_of_bounds_write("cart", addr, value);
            }
        }
        self.inner.sync_mappings(memory);
    }
}
