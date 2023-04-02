use crate::cartridge::{Cartridge};
use crate::mapper;
use crate::mapper::{NameTables, RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// Mapper 3: CNROM
/// https://www.nesdev.org/wiki/INES_Mapper_003
pub struct CNRomMapper {
    map: MemoryMap,
    nametables: NameTables,
}

impl CNRomMapper {
    pub fn new(cart: Cartridge) -> CNRomMapper {
        let mut map = MemoryMap::new(&cart);
        map.map_chr_8k(0);

        match cart.prg_rom.len() {
            0x4000 => { // 16KiB (mirrored into two)
                map.map_prg_16k(0, 0);
                map.map_prg_16k(1, 0);
            }
            0x8000 => {  // 32KiB
                map.map_prg_32k(0);
            }
            _ => panic!("PRG ROM should be 16KiB or 32KiB"),
        }

        CNRomMapper {
            map,
            nametables: NameTables::new(cart.mirroring),
        }
    }
}

impl RawMapper for CNRomMapper {
    fn write_main_bus(&mut self, _addr: u16, value: u8) {
        self.map.map_chr_8k(value as u8);
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        self.map.read_main_bus(addr)
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                self.map.access_ppu_bus(addr, value, write)
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
