use crate::cartridge::{Cartridge};
use crate::mapper;
use crate::mapper::{NameTables, RawMapper};

/// Mapper 2: UxROM
/// https://www.nesdev.org/wiki/UxROM
pub struct UxRomMapper {
    chr_ram: [u8; 8192],
    prg_rom: Box<[u8]>,
    nametables: NameTables,
    prg_bank_lo: u8,
}

const BANK_SIZE: usize = 16 * 1024;
const BANK_MASK: usize = BANK_SIZE - 1;

impl UxRomMapper {
    pub fn new(cart: Cartridge) -> UxRomMapper {
        UxRomMapper {
            prg_rom: cart.prg_rom.into_boxed_slice(),
            chr_ram: [0; 8192],
            nametables: NameTables::new(cart.mirroring),
            prg_bank_lo: 0,
        }
    }
}

impl RawMapper for UxRomMapper {
    fn write_main_bus(&mut self, _addr: u16, value: u8) {
        self.prg_bank_lo = value;
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                let base = self.prg_bank_lo as usize * BANK_SIZE;
                self.prg_rom[base..base+BANK_SIZE][addr as usize & BANK_MASK]
            }
            0xC000..=0xFFFF => {
                self.prg_rom[self.prg_rom.len() - BANK_SIZE..][addr as usize & BANK_MASK]
            }
            _ => {
                mapper::out_of_bounds_read("cart", addr)
            }
        }
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                if write {
                    self.chr_ram[addr as usize] = value;
                }
                self.chr_ram[addr as usize]
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
