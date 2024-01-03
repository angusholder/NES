use std::ops::Range;
use log::{warn};
use crate::cartridge::{Cartridge, CHR, NametableMirroring};
use crate::mapper;
use crate::mapper::nametables::NameTables;

pub struct MemoryMap {
    /// Covers 8 x 1K banks (0x400) between 0x0000 and 0x1FFF.
    chr_base_addrs: [usize; 8],
    /// Controls if chr_storage is RAM or ROM.
    chr_writeable: bool,
    /// RAM or ROM, depending on the cartridge.
    chr_storage: Box<[u8]>,

    /// Covers 4 x 8K banks (0x2000), between 0x8000 and 0xFFFF.
    prg_base_addrs: [usize; 4],
    prg_rom: Box<[u8]>,

    pub nametables: NameTables,
}

const PRG_PAGE: usize = 8 * 1024;
const CHR_PAGE: usize = 1024;

impl MemoryMap {
    pub fn new(cart: Cartridge) -> MemoryMap {
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
            base_addr = self.prg_rom_len() - base_addr;
        }
        base_addr %= self.prg_rom_len();

        for (i, bank) in banks.enumerate() {
            let bank = bank as usize;
            self.prg_base_addrs[bank] = base_addr + i*PRG_PAGE;
        }
    }

    pub fn map_chr_1k(&mut self, bank: usize, base_addr: usize) {
        self.chr_base_addrs[bank] = base_addr;
    }

    pub fn map_chr_2k(&mut self, bank: usize, base_addr: usize) {
        self.chr_base_addrs[bank+0] = base_addr + 0*CHR_PAGE;
        self.chr_base_addrs[bank+1] = base_addr + 1*CHR_PAGE;
    }

    pub fn map_chr_4k(&mut self, bank: usize, base_addr: usize) {
        self.chr_base_addrs[bank+0] = base_addr + 0*CHR_PAGE;
        self.chr_base_addrs[bank+1] = base_addr + 1*CHR_PAGE;
        self.chr_base_addrs[bank+2] = base_addr + 2*CHR_PAGE;
        self.chr_base_addrs[bank+3] = base_addr + 3*CHR_PAGE;
    }

    pub fn map_chr_8k(&mut self, base_addr: usize) {
        self.chr_base_addrs[0] = base_addr + 0*CHR_PAGE;
        self.chr_base_addrs[1] = base_addr + 1*CHR_PAGE;
        self.chr_base_addrs[2] = base_addr + 2*CHR_PAGE;
        self.chr_base_addrs[3] = base_addr + 3*CHR_PAGE;
        self.chr_base_addrs[4] = base_addr + 4*CHR_PAGE;
        self.chr_base_addrs[5] = base_addr + 5*CHR_PAGE;
        self.chr_base_addrs[6] = base_addr + 6*CHR_PAGE;
        self.chr_base_addrs[7] = base_addr + 7*CHR_PAGE;
    }
}

impl MemoryMap {
    /// [addr] expected to be in range 0x8000..0xFFFF
    pub fn read_prg(&self, addr: u16) -> u8 {
        let bank_no = (addr as usize >> 0x1FFFu32.count_ones()) & 3;
        let base_addr = self.prg_base_addrs[bank_no];
        self.prg_rom[base_addr + (addr as usize & 0x1FFF)]
    }

    pub fn read_pattern_table(&self, addr: u16) -> u8 {
        // The PPU address space is 14 bits - "Valid addresses are $0000–$3FFF; higher addresses will be mirrored down" - https://www.nesdev.org/wiki/PPU_registers#Address_($2006)_%3E%3E_write_x2
        let bank_no = (addr as usize >> 0x3FFu32.count_ones()) & 7;
        let base_addr = self.chr_base_addrs[bank_no];
        self.chr_storage[(base_addr + (addr as usize & 0x3FF)) % self.chr_storage.len()]
    }

    pub fn write_pattern_table(&mut self, addr: u16, value: u8) {
        // The PPU address space is 14 bits - "Valid addresses are $0000–$3FFF; higher addresses will be mirrored down" - https://www.nesdev.org/wiki/PPU_registers#Address_($2006)_%3E%3E_write_x2
        let bank_no = (addr as usize >> 0x3FFu32.count_ones()) & 7;
        let base_addr = self.chr_base_addrs[bank_no];

        if self.chr_writeable {
            self.chr_storage[base_addr + (addr as usize & 0x3FF)] = value;
        } else {
            mapper::out_of_bounds_write("CHR ROM", addr, value);
        }
    }
}
