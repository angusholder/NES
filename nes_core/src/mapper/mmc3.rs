use std::rc::Rc;
use log::{info};
use crate::cartridge::{NametableMirroring};
use crate::mapper;
use crate::mapper::{RawMapper};
use crate::mapper::memory_map::MemoryMap;
use crate::nes::{IRQSource, Signals};

pub struct MMC3Mapper {
    bank_reg: [u8; 8],
    // The next write to the bank data register will affect bank_reg[bank_reg_select]
    bank_reg_select: u8,
    prg_bank_mode: PRGBankMode,
    /// 0: two 2KB banks at $0000-$0FFF, four 1KB banks at $1000-$1FFF
    /// 1: two 2KB banks at $1000-$1FFF, four 1KB banks at $0000-$0FFF
    chr_a12_inversion: bool,

    irq_counter: u8,
    irq_counter_reload_value: u8,
    irq_counter_reload: bool,
    irq_enable: bool,

    signals: Rc<Signals>,
}

#[derive(Debug)]
enum PRGBankMode {
    /// $8000-$9FFF swappable, $C000-$DFFF fixed to second-last bank
    Swappable89 = 0,
    /// $C000-$DFFF swappable, $8000-$9FFF fixed to second-last bank
    SwappableCD = 1,
}

impl MMC3Mapper {
    pub fn new(signals: Rc<Signals>) -> MMC3Mapper {
        MMC3Mapper {
            bank_reg: [0; 8],
            bank_reg_select: 0,
            prg_bank_mode: PRGBankMode::Swappable89,
            chr_a12_inversion: false,

            irq_counter: 0,
            irq_counter_reload_value: 0,
            irq_counter_reload: false,
            irq_enable: false,

            signals,
        }
    }

    fn sync_mappings(&self, memory: &mut MemoryMap) {
        // Swap 0x0000-0x0FFF with 0x1000-0x1FFF
        let flip = if self.chr_a12_inversion { 4 } else { 0 };

        // CHR memory is 8 banks of 0x400/1KB each:
        // 0x0000-0x07FF
        memory.map_chr_1k(0 ^ flip, self.bank_reg[0]);
        memory.map_chr_1k(1 ^ flip, self.bank_reg[0]+1);
        // 0x0800-0x0FFF
        memory.map_chr_1k(2 ^ flip, self.bank_reg[1]);
        memory.map_chr_1k(3 ^ flip, self.bank_reg[1]+1);
        // 0x1000-0x13FF
        memory.map_chr_1k(4 ^ flip, self.bank_reg[2]);
        // 0x1400-0x17FF
        memory.map_chr_1k(5 ^ flip, self.bank_reg[3]);
        // 0x1800-0x1BFF
        memory.map_chr_1k(6 ^ flip, self.bank_reg[4]);
        // 0x1C00-0x1FFF
        memory.map_chr_1k(7 ^ flip, self.bank_reg[5]);

        match self.prg_bank_mode {
            PRGBankMode::Swappable89 => {
                memory.map_prg_8k(0, self.bank_reg[6] as i32); // R6
                memory.map_prg_8k(1, self.bank_reg[7] as i32); // R7
                memory.map_prg_8k(2, -2); // 2nd last page
                memory.map_prg_8k(3, -1); // Last page
            }
            PRGBankMode::SwappableCD => {
                memory.map_prg_8k(0, -2); // 2nd last page
                memory.map_prg_8k(1, self.bank_reg[7] as i32); // R7
                memory.map_prg_8k(2, self.bank_reg[6] as i32); // R6
                memory.map_prg_8k(3, -1); // Last page
            }
        }
    }

    fn write_register(&mut self, memory: &mut MemoryMap, addr: u16, value: u8) {
        match addr & 0xE001 {
            // Bank select
            0x8000 => {
                self.bank_reg_select = value & 0b111;
                info!("Selected R{}", self.bank_reg_select);
                self.chr_a12_inversion = value & 0x80 != 0;
                self.prg_bank_mode = if value & 0x40 == 0 { PRGBankMode::Swappable89 } else { PRGBankMode::SwappableCD };
                self.sync_mappings(memory);
            }
            // Bank data
            0x8001 => {
                let sel = self.bank_reg_select as usize;
                self.bank_reg[sel] = value;
                if sel == 0 || sel == 1 {
                    // Odd-numbered banks can't be selected by the 2KB bank slots.
                    self.bank_reg[sel] &= !1;
                }
                if sel == 6 || sel == 7 {
                    // There's only 6 PRG ROM address lines
                    self.bank_reg[sel] &= 0b0011_1111;
                }
                info!("R{sel} = {}", self.bank_reg[sel]);
                self.sync_mappings(memory);
            }
            // Mirroring
            0xA000 => {
                let mirroring = match value & 1 {
                    0 => NametableMirroring::Vertical,
                    1 => NametableMirroring::Horizontal,
                    _ => unreachable!(),
                };
                info!("{:?} mirroring", mirroring);
                memory.set_nametable_mirroring(mirroring);
            }
            // PRG RAM protect
            0xA001 => {
                // Not implemented
            }
            // IRQ latch
            0xC000 => {
                self.irq_counter_reload_value = value;
            }
            // IRQ reload
            0xC001 => {
                // Triggers the counter to load the reload value upon the next scanline cycle.
                self.irq_counter = 0;
            }
            // IRQ disable
            0xE000 => {
                self.irq_enable = false;
                self.signals.acknowledge_irq(IRQSource::MMC3);
            }
            // IRQ enable
            0xE001 => {
                self.irq_enable = true;
            }
            _ => unreachable!(),
        }
    }
}

impl RawMapper for MMC3Mapper {
    fn init_memory_map(&self, memory: &mut MemoryMap) {
        // Initialize prg_banks and chr_banks.
        self.sync_mappings(memory);
    }

    fn write_main_bus(&mut self, memory: &mut MemoryMap, addr: u16, value: u8) {
        match addr {
            0x8000..=0xFFFF => {
                self.write_register(memory, addr, value);
            }
            _ => {
                mapper::out_of_bounds_write("cart", addr, value)
            }
        }
    }

    fn on_cycle_scanline(&mut self) {
        if self.irq_counter == 0 && self.irq_enable {
            self.signals.request_irq(IRQSource::MMC3);
        }
        if self.irq_counter == 0 || self.irq_counter_reload {
            self.irq_counter = self.irq_counter_reload_value;
        } else {
            self.irq_counter -= 1;
        }
    }
}
