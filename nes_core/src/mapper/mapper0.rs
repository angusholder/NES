use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper;
use crate::mapper::RawMapper;

/// Mapper 0: NROM
/// https://www.nesdev.org/wiki/NROM
pub struct NROMMapper {
    /// 8KiB
    chr_rom: [u8; 8192],
    /// 16KiB or 32KiB
    prg_rom0: [u8; 16_384],
    prg_rom1: Option<[u8; 16_384]>,
    mirroring: NametableMirroring,
    nametables: [u8; 0x800],
}

impl NROMMapper {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            chr_rom: cart.chr_rom.try_into().expect("CHR ROM should be 8KiB"),
            prg_rom0: cart.prg_rom[..0x4000].try_into().unwrap(),
            prg_rom1: match cart.prg_rom.len() {
                0x4000 => None,
                0x8000 => Some(cart.prg_rom[0x4000..].try_into().unwrap()),
                _ => panic!("PRG ROM should be 16KiB or 32KiB"),
            },
            mirroring: cart.mirroring,
            nametables: [0; 0x800],
        }
    }
}

impl RawMapper for NROMMapper {
    fn access_main_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                if write {
                    mapper::out_of_bounds_write("PRG ROM", addr, value);
                }
                return self.prg_rom0[addr as usize - 0x8000];
            }
            0xC000..=0xFFFF => {
                if write {
                    mapper::out_of_bounds_write("PRG ROM", addr, value);
                }
                match self.prg_rom1 {
                    Some(prg_rom1) => return prg_rom1[addr as usize - 0xC000],
                    // Mirror of first 16KiB
                    None => self.prg_rom0[addr as usize - 0xC000]
                }
            }
            _ => {
                mapper::out_of_bounds_access("CPU memory space", addr, value, write)
            }
        }
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                if write {
                    mapper::out_of_bounds_write("CHR ROM", addr, value);
                }
                self.chr_rom[addr as usize]
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
