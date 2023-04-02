use crate::cartridge::{Cartridge};
use crate::mapper;
use crate::mapper::{NameTables, RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// Mapper 0: NROM
/// https://www.nesdev.org/wiki/NROM
pub struct NRomMapper {
    map: MemoryMap,
    nametables: NameTables,
}

impl NRomMapper {
    pub fn new(cart: Cartridge) -> Self {
        let mut map = MemoryMap::new(&cart);
        map.map_chr_8k(0);

        match cart.prg_rom.len() {
            // 16KiB (mirrored into two)
            0x4000 => {
                map.map_prg_16k(0, 0);
                map.map_prg_16k(1, 0);
            }
            // 32KiB
            0x8000 => {
                map.map_prg_32k(0);
            }
            _ => panic!("PRG ROM should be 16KiB or 32KiB"),
        }

        Self {
            map,
            nametables: NameTables::new(cart.mirroring),
        }
    }
}

impl RawMapper for NRomMapper {
    fn write_main_bus(&mut self, addr: u16, value: u8) {
        mapper::out_of_bounds_write("CPU memory space", addr, value);
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        self.map.read_main_bus(addr)
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF if !write => {
                self.map.read_ppu_bus(addr)
            },
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                self.nametables.access(addr, value, write)
            }
            _ => {
                mapper::out_of_bounds_access("PPU memory space", addr, value, write)
            }
        }
    }
}
