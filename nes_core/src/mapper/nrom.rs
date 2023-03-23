use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper;
use crate::mapper::RawMapper;

/// Mapper 0: NROM
/// https://www.nesdev.org/wiki/NROM
pub struct NRomMapper {
    /// 8KiB
    chr_rom: [u8; 8192],
    /// 32KiB, or 16KiB mirrored into two
    prg_rom: [u8; 32*1024],
    mirroring: NametableMirroring,
    nametables: [u8; 0x800],
}

impl NRomMapper {
    pub fn new(cart: Cartridge) -> Self {
        let mut prg_rom = [0u8; 32*1024];
        match cart.prg_rom.len() {
            // 16KiB (mirrored into two)
            0x4000 => {
                prg_rom[..16*1024].copy_from_slice(&cart.prg_rom);
                prg_rom[16*1024..].copy_from_slice(&cart.prg_rom);
            }
            // 32KiB
            0x8000 => {
                prg_rom.copy_from_slice(&cart.prg_rom);
            }
            _ => panic!("PRG ROM should be 16KiB or 32KiB"),
        }

        Self {
            chr_rom: cart.chr_rom.try_into().expect("CHR ROM should be 8KiB"),
            prg_rom,
            mirroring: cart.mirroring,
            nametables: [0; 0x800],
        }
    }
}

impl RawMapper for NRomMapper {
    fn write_main_bus(&mut self, addr: u16, value: u8) {
        mapper::out_of_bounds_write("CPU memory space", addr, value);
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                self.prg_rom[addr as usize - 0x8000]
            }
            _ => {
                mapper::out_of_bounds_read("CPU memory space", addr)
            }
        }
    }

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF if !write => {
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
