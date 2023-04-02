use crate::cartridge::{Cartridge};
use crate::mapper;
use crate::mapper::{NameTables, RawMapper};
use crate::mapper::memory_map::MemoryMap;

/// Mapper 2: UxROM
/// https://www.nesdev.org/wiki/UxROM
pub struct UxRomMapper {
    map: MemoryMap,
    nametables: NameTables,
}

const BANK_SIZE: usize = 16 * 1024;
const BANK_MASK: usize = BANK_SIZE - 1;

impl UxRomMapper {
    pub fn new(cart: Cartridge) -> UxRomMapper {
        let mut map = MemoryMap::new(&cart);
        map.map_chr_8k(0);
        map.map_prg_16k(0, 0);
        map.map_prg_16k(1, -1);
        map.configure_chr_ram(8192);
        UxRomMapper {
            map,
            nametables: NameTables::new(cart.mirroring),
        }
    }
}

impl RawMapper for UxRomMapper {
    fn write_main_bus(&mut self, _addr: u16, value: u8) {
        self.map.map_prg_16k(0, value as i32);
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
