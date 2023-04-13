use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper::memory_map::MemoryMap;
use crate::nes::Signals;

mod nrom;
mod mmc1;
mod uxrom;
mod cnrom;
mod mmc2;
mod mmc3;
mod memory_map;
mod axrom;
mod dxrom;

const DEBUG_MAPPINGS: bool = false;

/// The mapper covers two address spaces - the CPU memory map, and the PPU memory map.
/// The CPU memory map is 16-bit, and the PPU memory map is 14-bit.
///
/// There's only one method for each address space, and the `write` parameter tells us whether we're
/// reading or writing (so we don't have to duplicate the address logic between reads and writes).
pub trait RawMapper : Any {
    fn init_memory_map(&self, memory: &mut MemoryMap);

    fn write_main_bus(&mut self, memory: &mut MemoryMap, addr: u16, value: u8);

    /// Returns a callback to be invoked after reading the PPU pattern table.
    fn get_ppu_pattern_post_read_hook(&self) -> Option<Rc<PPUPatternPostReadHook>> { None }

    fn on_cycle_scanline(&mut self) {}
}

/// A callback to invoke after reading the PPU pattern table.
pub type PPUPatternPostReadHook = dyn Fn(&mut MemoryMap, u16);

#[derive(Copy, Clone)]
pub struct MapperDescriptor {
    pub number: u32,
    pub name: &'static str,
    pub new_mapper: fn(Rc<Signals>) -> Rc<RefCell<dyn RawMapper>>,
}

static DESCRIPTORS: &[MapperDescriptor] = &[
    MapperDescriptor::NROM,
    MapperDescriptor::MMC1,
    MapperDescriptor::UxROM,
    MapperDescriptor::CNROM,
    MapperDescriptor::MMC3,
    MapperDescriptor::AxROM,
    MapperDescriptor::MMC2,
    MapperDescriptor::DxROM,
];

fn wrap(raw_mapper: impl RawMapper) -> Rc<RefCell<dyn RawMapper>> {
    Rc::new(RefCell::new(raw_mapper))
}

#[allow(non_upper_case_globals)]
impl MapperDescriptor {
    pub fn for_number(number: u32) -> Option<MapperDescriptor> {
        for desc in DESCRIPTORS {
            if number == desc.number {
                return Some(*desc);
            }
        }
        None
    }

    pub const NROM: MapperDescriptor = MapperDescriptor {
        number: 0,
        name: "NROM",
        new_mapper: |_| wrap(nrom::NRomMapper::new()),
    };
    pub const MMC1: MapperDescriptor = MapperDescriptor {
        number: 1,
        name: "MMC1",
        new_mapper: |_| wrap(mmc1::MMC1Mapper::new()),
    };
    pub const UxROM: MapperDescriptor = MapperDescriptor {
        number: 2,
        name: "UxROM",
        new_mapper: |_| wrap(uxrom::UxRomMapper::new()),
    };
    pub const CNROM: MapperDescriptor = MapperDescriptor {
        number: 3,
        name: "CNROM",
        new_mapper: |_| wrap(cnrom::CNRomMapper::new()),
    };
    pub const MMC3: MapperDescriptor = MapperDescriptor {
        number: 4,
        name: "MMC3",
        new_mapper: |signals| wrap(mmc3::MMC3Mapper::new(signals.clone())),
    };
    pub const AxROM: MapperDescriptor = MapperDescriptor {
        number: 7,
        name: "AxROM",
        new_mapper: |_| wrap(axrom::AxRomMapper::new()),
    };
    pub const MMC2: MapperDescriptor = MapperDescriptor {
        number: 9,
        name: "MMC2",
        new_mapper: |_| wrap(mmc2::MMC2Mapper::new()),
    };
    pub const DxROM: MapperDescriptor = MapperDescriptor {
        number: 206,
        name: "DxROM/Tengen MIMIC-1/Namcot 118",
        new_mapper: |_| wrap(dxrom::DxROMMapper::new()),
    };
}

#[derive(Clone)]
pub struct Mapper {
    raw_mapper: Rc<RefCell<dyn RawMapper>>,
    memory_map: Rc<RefCell<MemoryMap>>,
    ppu_pattern_post_read_hook: Option<Rc<PPUPatternPostReadHook>>,
}

impl Mapper {
    pub fn new(cart: Cartridge, signals: Rc<Signals>) -> Mapper {
        let raw_mapper: Rc<RefCell<dyn RawMapper>> = (cart.mapper_descriptor.new_mapper)(signals);

        let memory_map = Rc::new(RefCell::new(MemoryMap::new(cart)));

        raw_mapper.borrow_mut().init_memory_map(&mut memory_map.borrow_mut());

        let ppu_pattern_post_read_hook: Option<Rc<PPUPatternPostReadHook>> = raw_mapper.borrow_mut().get_ppu_pattern_post_read_hook();

        Mapper {
            raw_mapper,
            memory_map,
            ppu_pattern_post_read_hook,
        }
    }

    pub fn read_main_bus(&mut self, addr: u16) -> u8 {
        self.memory_map.borrow().read_main_bus(addr)
    }

    pub fn write_main_bus(&mut self, addr: u16, value: u8) {
        self.memory_map.borrow_mut().write_main_bus(&mut *self.raw_mapper.borrow_mut(), addr, value);
    }

    pub fn read_ppu_bus(&mut self, addr: u16) -> u8 {
        let result = self.memory_map.borrow().read_ppu_bus(mask_ppu_addr(addr));
        if let Some(post_read_hook) = self.ppu_pattern_post_read_hook.as_ref() {
            post_read_hook(&mut self.memory_map.borrow_mut(), addr);
        }
        result
    }

    pub fn write_ppu_bus(&mut self, addr: u16, value: u8) {
        self.memory_map.borrow_mut().write_ppu_bus(mask_ppu_addr(addr), value);
    }

    pub fn on_cycle_scanline(&mut self) {
        self.raw_mapper.borrow_mut().on_cycle_scanline();
    }
}

/// The PPU address space is 14 bits, but the CPU address space is 16 bits.
/// "Valid addresses are $0000–$3FFF; higher addresses will be mirrored down" - https://www.nesdev.org/wiki/PPU_registers#Address_($2006)_%3E%3E_write_x2
#[inline(always)]
fn mask_ppu_addr(addr: u16) -> u16 {
    addr & 0x3FFF
}

/// See https://www.nesdev.org/wiki/Mirroring#Nametable_Mirroring
pub struct NameTables {
    storage: [u8; 0x1000],
    base_addrs: [NtAddr; 4],
}

// This is an enum so the compiler can omit the bounds check when accessing `NameTables.storage`.
#[derive(Clone, Copy)]
enum NtAddr {
    Addr000 = 0x000,
    Addr400 = 0x400,
    Addr800 = 0x800,
    AddrC00 = 0xC00,
}

const NT_2000: usize = 0;
const NT_2400: usize = 1;
const NT_2800: usize = 2;
const NT_2C00: usize = 3;

impl NameTables {
    pub fn new(mirroring: NametableMirroring) -> NameTables {
        use crate::mapper::NtAddr::*;
        let mut nt = NameTables {
            storage: [0; 0x1000],
            base_addrs: [Addr000, Addr000, Addr000, Addr000],
        };
        nt.update_mirroring(mirroring);
        nt
    }

    pub fn update_mirroring(&mut self, mirroring: NametableMirroring) {
        use crate::mapper::NtAddr::*;

        match mirroring {
            NametableMirroring::Horizontal => {
                self.base_addrs[NT_2000] = Addr000;
                self.base_addrs[NT_2400] = Addr000;
                self.base_addrs[NT_2800] = Addr400;
                self.base_addrs[NT_2C00] = Addr400;
            },
            NametableMirroring::Vertical => {
                self.base_addrs[NT_2000] = Addr000;
                self.base_addrs[NT_2400] = Addr400;
                self.base_addrs[NT_2800] = Addr000;
                self.base_addrs[NT_2C00] = Addr400;
            },
            NametableMirroring::SingleScreenLowerBank => {
                self.base_addrs = [Addr000, Addr000, Addr000, Addr000];
            }
            NametableMirroring::SingleScreenUpperBank => {
                self.base_addrs = [Addr400, Addr400, Addr400, Addr400];
            }
            NametableMirroring::FourScreen => {
                self.base_addrs[NT_2000] = Addr000;
                self.base_addrs[NT_2400] = Addr400;
                self.base_addrs[NT_2800] = Addr800;
                self.base_addrs[NT_2C00] = AddrC00;
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let offset: NtAddr = self.base_addrs[Self::addr_to_offset(addr)];
        self.storage[offset as usize + (addr as usize & 0x3FF)]
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        let offset: NtAddr = self.base_addrs[Self::addr_to_offset(addr)];
        self.storage[offset as usize + (addr as usize & 0x3FF)] = value;
    }

    #[inline(always)]
    fn addr_to_offset(addr: u16) -> usize {
        match addr & 0xC00 {
            0x000 => NT_2000,
            0x400 => NT_2400,
            0x800 => NT_2800,
            0xC00 => NT_2C00,
            _ => unreachable!()
        }
    }
}

#[test]
fn test_nametable_addr_to_offset() {
    assert_eq!(NameTables::addr_to_offset(0x2000), NT_2000);
    assert_eq!(NameTables::addr_to_offset(0x23FF), NT_2000);
    assert_eq!(NameTables::addr_to_offset(0x2400), NT_2400);
    assert_eq!(NameTables::addr_to_offset(0x27FF), NT_2400);
    assert_eq!(NameTables::addr_to_offset(0x2800), NT_2800);
    assert_eq!(NameTables::addr_to_offset(0x2BFF), NT_2800);
    assert_eq!(NameTables::addr_to_offset(0x2C00), NT_2C00);
    assert_eq!(NameTables::addr_to_offset(0x2FFF), NT_2C00);
}

#[inline(never)]
#[track_caller]
#[cold]
pub fn out_of_bounds_read(context: &str, addr: u16) -> u8 {
    if DEBUG_MAPPINGS {
        log::warn!("Attempted to read {context} out of bounds at {addr:04X}");
    }

    return 0;
}

#[inline(never)]
#[track_caller]
#[cold]
pub fn out_of_bounds_write(context: &str, addr: u16, value: u8) {
    if DEBUG_MAPPINGS {
        log::warn!("Attempted to write {context} out of bounds at {addr:04X} with {value} (0x{value:02X})");
    }
}
