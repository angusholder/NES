use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper;
use crate::mapper::RawMapper;

/// Mapper 3: CNROM
/// https://www.nesdev.org/wiki/INES_Mapper_003
pub struct CNRomMapper {
    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,
    mirroring: NametableMirroring,
    nametables: [u8; 0x800],
    chr_bank: u8,
}

impl CNRomMapper {
    pub fn new(cart: Cartridge) -> CNRomMapper {
        CNRomMapper {
            prg_rom: cart.prg_rom.into_boxed_slice(),
            chr_rom: cart.chr_rom.into_boxed_slice(),
            mirroring: cart.mirroring,
            nametables: [0; 0x800],
            chr_bank: 0,
        }
    }
}

const CHR_BANK_SIZE: usize = 8 * 1024;

impl RawMapper for CNRomMapper {
    fn write_main_bus(&mut self, _addr: u16, value: u8) {
        self.chr_bank = value;
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                self.prg_rom[(addr & 0x7FFF) as usize % self.prg_rom.len()]
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
                    mapper::out_of_bounds_write("CHR", addr, value);
                }
                let base = self.chr_bank as usize * CHR_BANK_SIZE;
                self.chr_rom[base..base + CHR_BANK_SIZE][addr as usize]
            },
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                mapper::access_nametable(&mut self.nametables, self.mirroring, addr & 0x2FFF, value, write)
            }
            _ => {
                mapper::out_of_bounds_access("PPU memory space", addr, value, write)
            }
        }
    }
}
