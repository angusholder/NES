use crate::mapper;
use crate::mapper::{RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// Mapper 0: NROM
/// https://www.nesdev.org/wiki/NROM
pub struct NRomMapper {
}

impl NRomMapper {
    pub fn new() -> Self {
        NRomMapper {
        }
    }
}

impl RawMapper for NRomMapper {
    fn init(&mut self, memory: &mut MemoryMap) {
        memory.map_chr_8k(0);

        match memory.prg_rom_len() {
            // 16KiB (mirrored into two)
            0x4000 => {
                memory.map_prg_16k(0, 0);
                memory.map_prg_16k(1, 0);
            }
            // 32KiB
            0x8000 => {
                memory.map_prg_32k(0);
            }
            _ => panic!("PRG ROM should be 16KiB or 32KiB"),
        }
    }

    fn write_main_bus(&mut self, _memory: &mut MemoryMap, addr: u16, value: u8) {
        mapper::out_of_bounds_write("CPU memory space", addr, value);
    }
}
