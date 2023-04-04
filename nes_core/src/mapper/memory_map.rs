use std::ops::Range;
use log::{warn};
use crate::cartridge::{Cartridge, CHR, NametableMirroring};
use crate::mapper;
use crate::mapper::{NameTables, RawMapper};

pub struct MemoryMap {
    /// Covers 8 x 1K banks (0x400) between 0x0000 and 0x1FFF.
    chr_base_addrs: [usize; 8],
    /// Controls if chr_storage is RAM or ROM.
    chr_writeable: bool,
    /// RAM or ROM, depending on the cartridge.
    chr_storage: Box<[u8]>,

    /// 0x6000-0x7FFF
    wram: Option<Box<[u8; 0x2000]>>,

    /// Covers 4 x 8K banks (0x2000), between 0x8000 and 0xFFFF.
    prg_base_addrs: [usize; 4],
    prg_rom: Box<[u8]>,

    nametables: NameTables,
}

const PRG_PAGE: usize = 8 * 1024;
const CHR_PAGE: usize = 1024;

impl MemoryMap {
    pub fn new(cart: Cartridge) -> MemoryMap {
        let wram = match cart.prg_ram_size {
            8192 => Some(Box::new([0; 8192])),
            0 => None,
            other => {
                warn!("Unexpected PRG RAM size {other}");
                None
            }
        };
        if cart.prg_ram_battery_backed {
            warn!("Battery-backed PRG RAM is not supported.");
        }

        MemoryMap {
            chr_base_addrs: [0; 8],
            chr_writeable: matches!(cart.chr, CHR::RAM(_)),
            chr_storage: match cart.chr {
                CHR::RAM(ram_size) => vec![0; ram_size].into_boxed_slice(),
                CHR::ROM(rom) => rom,
            },

            wram,
            
            prg_base_addrs: [0; 4],
            prg_rom: cart.prg_rom.into_boxed_slice(),

            nametables: NameTables::new(cart.mirroring),
        }
    }

    pub fn prg_rom_len(&self) -> usize { self.prg_rom.len() }

    pub fn set_nametable_mirroring(&mut self, mirroring: NametableMirroring) {
        self.nametables.update_mirroring(mirroring);
    }
    
    pub fn map_prg_32k(&mut self, page_index: i32) {
        self.map_prg_range(0..4, page_index, 32 * 1024);
    }

    pub fn map_prg_16k(&mut self, bank: u8, page_index: i32) {
        assert!(bank < 2);
        self.map_prg_range(bank*2..(bank+1)*2, page_index, 16 * 1024);
    }

    pub fn map_prg_8k(&mut self, bank: u8, page_index: i32) {
        assert!(bank < 4);
        self.map_prg_range(bank..bank+1, page_index, 8 * 1024);
    }

    fn map_prg_range(&mut self, banks: Range<u8>, page_index: i32, page_size: usize) {
        let mut base_addr: usize = page_index.unsigned_abs() as usize * page_size;
        if page_index < 0 {
            base_addr = self.prg_rom.len() - base_addr;
        }

        for (i, bank) in banks.enumerate() {
            let bank = bank as usize;
            self.prg_base_addrs[bank] = base_addr + i*PRG_PAGE;
        }
    }

    pub fn map_chr_1k(&mut self, bank: u8, page_index: u8) {
        assert!(bank < 8);
        self.map_chr_range(bank..bank+1, page_index, 1024);
    }

    pub fn map_chr_2k(&mut self, bank: u8, page_index: u8) {
        assert!(bank < 4);
        self.map_chr_range(bank*2..(bank+1)*2, page_index, 2048);
    }

    pub fn map_chr_4k(&mut self, bank: u8, page_index: u8) {
        assert!(bank < 2);
        self.map_chr_range(bank*4..(bank+1)*4, page_index, 4096);
    }

    pub fn map_chr_8k(&mut self, page_index: u8) {
        self.map_chr_range(0..8, page_index, 8192);
    }

    fn map_chr_range(&mut self, banks: Range<u8>, page_index: u8, page_size: usize) {
        let base_addr: usize = page_index as usize * page_size;
        for (i, bank) in banks.enumerate() {
            let bank = bank as usize;
            self.chr_base_addrs[bank] = base_addr + i*CHR_PAGE;
        }
    }
}

impl MemoryMap {
    pub fn read_main_bus(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let bank_no = (addr as usize >> 0x1FFFu32.count_ones()) & 3;
                let base_addr = self.prg_base_addrs[bank_no];
                self.prg_rom[base_addr + (addr as usize & 0x1FFF)]
            }
            0x6000..=0x7FFF => {
                if let Some(wram) = self.wram.as_ref() {
                    wram[addr as usize & 0x1FFF]
                } else {
                    mapper::out_of_bounds_read("WRAM", addr)
                }
            }
            _ => {
                mapper::out_of_bounds_read("CPU memory space", addr)
            }
        }
    }

    pub(in crate::mapper) fn write_main_bus(&mut self, mapper: &mut dyn RawMapper, addr: u16, value: u8) {
        match addr {
            0x8000..=0xFFFF => {
                mapper.write_main_bus(self, addr, value);
            }
            0x6000..=0x7FFF => {
                if let Some(wram) = self.wram.as_mut() {
                    wram[addr as usize & 0x1FFF] = value;
                } else {
                    mapper::out_of_bounds_write("WRAM", addr, value);
                }
            }
            _ => {
                mapper::out_of_bounds_write("CPU memory space", addr, value);
            }
        }
    }

    pub fn read_ppu_bus(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let bank_no = (addr as usize >> 0x3FFu32.count_ones()) & 7;
                let base_addr = self.chr_base_addrs[bank_no];
                self.chr_storage[base_addr + (addr as usize & 0x3FF)]
            }
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                self.nametables.read(addr)
            }
            _ => {
                mapper::out_of_bounds_read("PPU memory space", addr)
            }
        }
    }

    pub fn write_ppu_bus(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let bank_no = (addr as usize >> 0x3FFu32.count_ones()) & 7;
                let base_addr = self.chr_base_addrs[bank_no];

                if self.chr_writeable {
                    self.chr_storage[base_addr + (addr as usize & 0x3FF)] = value;
                } else {
                    mapper::out_of_bounds_write("CHR ROM", addr, value);
                }
            }
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                self.nametables.write(addr, value);
            }
            _ => {
                mapper::out_of_bounds_write("PPU memory space", addr, value)
            }
        }
    }
}
