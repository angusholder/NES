use crate::mapper::{RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// Mapper 2: UxROM
/// https://www.nesdev.org/wiki/UxROM
pub struct UxRomMapper {
}

impl UxRomMapper {
    pub fn new() -> UxRomMapper {
        UxRomMapper {
        }
    }
}

impl RawMapper for UxRomMapper {
    fn init_memory_map(&self, memory: &mut MemoryMap) {
        memory.map_chr_8k(0);
        memory.map_prg_16k(0, 0);
        memory.map_prg_16k(1, -1);
    }

    fn write_main_bus(&mut self, memory: &mut MemoryMap, _addr: u16, value: u8) {
        memory.map_prg_16k(0, value as i32);
    }
}
