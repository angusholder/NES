use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use crate::cartridge::Cartridge;
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
    pub new_mapper: fn(Rc<Signals>) -> Box<RefCell<dyn RawMapper>>,
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

fn wrap(raw_mapper: impl RawMapper) -> Box<RefCell<dyn RawMapper>> {
    Box::new(RefCell::new(raw_mapper))
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
        new_mapper: |signals| wrap(mmc3::MMC3Mapper::new(signals)),
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

pub struct Mapper {
    raw_mapper: Box<RefCell<dyn RawMapper>>,
    memory_map: RefCell<MemoryMap>,
    ppu_pattern_post_read_hook: Option<Rc<PPUPatternPostReadHook>>,
    /// 0x6000-0x7FFF
    wram: Box<[Cell<u8>; 0x2000]>,
}

impl Mapper {
    pub fn new(cart: Cartridge, signals: Rc<Signals>) -> Mapper {
        let raw_mapper: Box<RefCell<dyn RawMapper>> = (cart.mapper_descriptor.new_mapper)(signals);

        let memory_map = RefCell::new(MemoryMap::new(cart));

        raw_mapper.borrow_mut().init_memory_map(&mut memory_map.borrow_mut());

        let ppu_pattern_post_read_hook: Option<Rc<PPUPatternPostReadHook>> = raw_mapper.borrow_mut().get_ppu_pattern_post_read_hook();

        const U8_0: Cell<u8> = Cell::new(0);
        Mapper {
            raw_mapper,
            memory_map,
            ppu_pattern_post_read_hook,
            wram: Box::new([U8_0; 0x2000]),
        }
    }

    pub fn read_main_bus(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                self.memory_map.borrow().read_prg(addr)
            }
            0x6000..=0x7FFF => {
                self.wram[addr as usize & 0x1FFF].get()
            }
            _ => {
                out_of_bounds_read("CPU memory space", addr)
            }
        }
    }

    pub fn write_main_bus(&self, addr: u16, value: u8) {
        match addr {
            0x8000..=0xFFFF => {
                self.raw_mapper.borrow_mut().write_main_bus(&mut self.memory_map.borrow_mut(), addr, value);
            }
            0x6000..=0x7FFF => {
                self.wram[addr as usize & 0x1FFF].set(value);
            }
            _ => {
                out_of_bounds_write("CPU memory space", addr, value);
            }
        }
    }

    #[inline(always)]
    pub fn read_nametable(&self, addr: u16) -> u8 {
        self.memory_map.borrow().read_nametable(addr)
    }

    #[inline(always)]
    pub fn read_pattern_table(&self, addr: u16) -> u8 {
        let result = self.memory_map.borrow().read_pattern_table(addr);
        if let Some(post_read_hook) = self.ppu_pattern_post_read_hook.as_ref() {
            post_read_hook(&mut self.memory_map.borrow_mut(), addr);
        }
        result
    }

    pub fn write_nametable(&self, addr: u16, value: u8) {
        self.memory_map.borrow_mut().write_nametable(addr, value);
    }

    pub fn write_pattern_table(&self, addr: u16, value: u8) {
        self.memory_map.borrow_mut().write_pattern_table(addr, value);
    }

    pub fn on_cycle_scanline(&self) {
        self.raw_mapper.borrow_mut().on_cycle_scanline();
    }
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
