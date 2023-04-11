use crate::mapper;
use crate::mapper::{RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// https://www.nesdev.org/wiki/INES_Mapper_206
/// Known as DxROM/Tengen MIMIC-1/Namcot 118
pub struct DxROMMapper {
    bank_reg: [u8; 8],
    // The next write to the bank data register will affect bank_reg[bank_reg_select]
    bank_reg_select: u8,
}

impl DxROMMapper {
    pub fn new() -> DxROMMapper {
        DxROMMapper {
            bank_reg: [0; 8],
            bank_reg_select: 0,
        }
    }

    fn sync_mappings(&self, memory: &mut MemoryMap) {
        // CHR memory is 8 banks of 0x400/1KB each:
        // 0x0000-0x07FF
        memory.map_chr_2k(0, self.bank_reg[0]);
        // 0x0800-0x0FFF
        memory.map_chr_2k(1, self.bank_reg[1]);
        // 0x1000-0x13FF
        memory.map_chr_1k(4, self.bank_reg[2]);
        // 0x1400-0x17FF
        memory.map_chr_1k(5, self.bank_reg[3]);
        // 0x1800-0x1BFF
        memory.map_chr_1k(6, self.bank_reg[4]);
        // 0x1C00-0x1FFF
        memory.map_chr_1k(7, self.bank_reg[5]);

        memory.map_prg_8k(0, self.bank_reg[6] as i32); // R6
        memory.map_prg_8k(1, self.bank_reg[7] as i32); // R7
        memory.map_prg_8k(2, -2); // 2nd last page
        memory.map_prg_8k(3, -1); // Last page
    }

    fn write_register(&mut self, memory: &mut MemoryMap, addr: u16, value: u8) {
        match addr & 0xE001 {
            // Bank select
            0x8000 => {
                self.bank_reg_select = value & 0b111;
            }
            // Bank data
            0x8001 => {
                let sel = self.bank_reg_select as usize;
                self.bank_reg[sel] = match sel {
                    // Odd-numbered banks can't be selected by the 2KB bank slots.
                    0 | 1 => value & 0b0011_1110,
                    2..=5 => value & 0b0011_1111,
                    // There's only 4 PRG ROM address lines
                    6 | 7 => value & 0b0000_1111,
                    _ => unreachable!(),
                };

                self.sync_mappings(memory);
            }
            _ => mapper::out_of_bounds_write("Cart", addr, value),
        }
    }
}

impl RawMapper for DxROMMapper {
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
}
