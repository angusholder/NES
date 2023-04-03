use crate::mapper::{RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// Mapper 3: CNROM
/// https://www.nesdev.org/wiki/INES_Mapper_003
pub struct CNRomMapper {
}

impl CNRomMapper {
    pub fn new() -> CNRomMapper {
        CNRomMapper {
        }
    }
}

impl RawMapper for CNRomMapper {
    fn init_memory_map(&self, memory: &mut MemoryMap) {
        memory.map_chr_8k(0);

        match memory.prg_rom_len() {
            0x4000 => { // 16KiB (mirrored into two)
                memory.map_prg_16k(0, 0);
                memory.map_prg_16k(1, 0);
            }
            0x8000 => {  // 32KiB
                memory.map_prg_32k(0);
            }
            _ => panic!("PRG ROM should be 16KiB or 32KiB"),
        }
    }

    fn write_main_bus(&mut self, map: &mut MemoryMap, _addr: u16, value: u8) {
        map.map_chr_8k(value);
    }
}
