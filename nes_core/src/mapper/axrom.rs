use crate::cartridge::NametableMirroring;
use crate::mapper::memory_map::MemoryMap;
use crate::mapper::RawMapper;

pub struct AxRomMapper {
    prg_bank: u8,
    // Either SingleScreenLowerBank or SingleScreenUpperBank
    mirroring: NametableMirroring,
}

impl AxRomMapper {
    pub fn new() -> AxRomMapper {
        AxRomMapper {
            prg_bank: 0,
            mirroring: NametableMirroring::SingleScreenLowerBank,
        }
    }

    pub fn sync_mapping(&self, memory: &mut MemoryMap) {
        memory.set_nametable_mirroring(self.mirroring);
        memory.map_prg_32k(self.prg_bank as i32);
    }
}

impl RawMapper for AxRomMapper {
    fn init_memory_map(&self, memory: &mut MemoryMap) {
        memory.configure_chr_ram(8192);
        memory.map_chr_8k(0);
        self.sync_mapping(memory);
    }

    fn write_main_bus(&mut self, memory: &mut MemoryMap, _addr: u16, value: u8) {
        self.prg_bank = value & 0b111;
        self.mirroring = match value >> 4 & 1 {
            0 => NametableMirroring::SingleScreenLowerBank,
            1 => NametableMirroring::SingleScreenUpperBank,
            _ => unreachable!(),
        };
        self.sync_mapping(memory);
    }
}
