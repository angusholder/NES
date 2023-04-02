use std::ops::Range;
use log::warn;
use crate::cartridge::Cartridge;
use crate::mapper;

type ReadHook = dyn Fn(&mut MemoryMap, u16) -> ();
type WriteHook = dyn Fn(&mut MemoryMap, u16, u8) -> ();

pub struct MemoryMap {
    /// Covers 8 x 1K banks (0x400) between 0x0000 and 0x1FFF.
    chr_base_addrs: [usize; 8],
    /// Controls if chr_storage is RAM or ROM.
    chr_writeable: bool,
    /// RAM or ROM, depending on the cartridge.
    chr_storage: Box<[u8]>,
    chr_read_hook: Option<Box<ReadHook>>,
    chr_write_hook: Option<Box<WriteHook>>,

    /// 0x6000-0x7FFF
    wram: Option<Box<[u8; 0x2000]>>,

    /// Covers 4 x 8K banks (0x2000), between 0x8000 and 0xFFFF.
    prg_base_addrs: [usize; 4],
    prg_rom: Box<[u8]>,
}

const PRG_PAGE: usize = 8 * 1024;
const CHR_PAGE: usize = 1024;

static PRG_BANK_NAMES: [&str; 4] = ["0x8000", "0xA000", "0xC000", "0xE000"];

impl MemoryMap {
    pub fn new(cart: &Cartridge) -> MemoryMap {
        let wram = match cart.prg_ram_size {
            8192 => Some(Box::new([0; 8192])),
            0 => None,
            other => {
                warn!("Unexpected PRG RAM size {other}");
                None
            }
        };

        MemoryMap {
            chr_base_addrs: [0; 8],
            chr_writeable: false,
            chr_storage: cart.chr_rom.clone().into_boxed_slice(),
            chr_read_hook: None,
            chr_write_hook: None,

            wram,
            
            prg_base_addrs: [0; 4],
            prg_rom: cart.prg_rom.clone().into_boxed_slice(),
        }
    }
    
    pub fn map_prg_32k(&mut self, page_index: i32) {
        self.map_prg_range(0..4, page_index, 32 * 1024);
    }

    pub fn map_prg_16k(&mut self, bank: u8, page_index: i32) {
        assert!(bank < 2);
        println!("map_prg_16k({bank}, {page_index})");
        self.map_prg_range(bank*2..(bank+1)*2, page_index, 16 * 1024);
    }

    pub fn map_prg_8k(&mut self, bank: u8, page_index: i32) {
        assert!(bank < 4);
        self.map_prg_range(bank..bank+1, page_index, 8 * 1024);
    }

    fn map_prg_range(&mut self, banks: Range<u8>, page_index: i32, page_size: usize) {
        let mut base_addr: usize = page_index.abs() as usize * page_size;
        if page_index < 0 {
            base_addr = self.prg_rom.len() - base_addr;
        }

        for (i, bank) in banks.enumerate() {
            let bank = bank as usize;
            self.prg_base_addrs[bank as usize] = base_addr + i*PRG_PAGE;
            println!("Mapped {} to {:05X}", PRG_BANK_NAMES[bank], self.prg_base_addrs[bank]);
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
        for i in banks {
            let i = i as usize;
            self.chr_base_addrs[i] = base_addr + i*CHR_PAGE;
        }
    }

    pub fn configure_chr_ram(&mut self, size: usize) {
        self.chr_storage = vec![0; size].into_boxed_slice();
        self.chr_writeable = true;
    }
}

impl MemoryMap {
    pub fn read_main_bus(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                if let Some(wram) = self.wram.as_ref() {
                    wram[addr as usize & 0x1FFF]
                } else {
                    mapper::out_of_bounds_read("WRAM", addr)
                }
            }
            0x8000..=0xFFFF => {
                let bank_no = (addr as usize >> 0x1FFFu32.count_ones()) & 3;
                let base_addr = self.prg_base_addrs[bank_no];
                self.prg_rom[base_addr + (addr as usize & 0x1FFF)]
            }
            _ => {
                mapper::out_of_bounds_read("CPU memory space", addr)
            }
        }
    }

    pub fn write_main_bus(&mut self, addr: u16, value: u8) {
        match addr {
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
        self.access_ppu_bus(addr, 0, false)
    }

    pub fn write_ppu_bus(&mut self, addr: u16, value: u8) {
        self.access_ppu_bus(addr, value, true);
    }

    pub fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let bank_no = (addr as usize >> 0x3FFu32.count_ones()) & 7;
                let base_addr = self.chr_base_addrs[bank_no];
                let ptr = &mut self.chr_storage[base_addr + (addr as usize & 0x3FF)];

                if write {
                    if self.chr_writeable {
                        *ptr = value;
                    } else {
                        mapper::out_of_bounds_write("CHR ROM", addr, value);
                    }
                }
                *ptr
            },
            // TODO: Nametable will be here too eventually.
            _ => {
                mapper::out_of_bounds_access("PPU memory space", addr, value, write)
            }
        }
    }
}
