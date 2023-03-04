use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::Cartridge;

mod mapper0;

/// The mapper covers two address spaces - the CPU memory map, and the PPU memory map.
/// The CPU memory map is 16-bit, and the PPU memory map is 14-bit.
///
/// There's only one method for each address space, and the `write` parameter tells us whether we're
/// reading or writing (so we don't have to duplicate the address logic between reads and writes).
trait RawMapper {
    fn access_main_bus(&mut self, addr: u16, value: u8, write: bool) -> u8;

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8;
}

#[derive(Clone)]
pub struct Mapper {
    mapper: Rc<RefCell<dyn RawMapper>>,
}

impl Mapper {
    pub fn new(cart: Cartridge) -> Result<Mapper, String> {
        Ok(match cart.mapper_num {
            0 => Mapper::wrap(mapper0::NROMMapper::new(cart)),
            _ => {
                return Err(format!("Unsupported mapper: {}", cart.mapper_num))
            }
        })
    }

    fn wrap<M: RawMapper + 'static>(raw_mapper: M) -> Mapper {
        Mapper {
            mapper: Rc::new(RefCell::new(raw_mapper)),
        }
    }

    pub fn read_main_bus(&mut self, addr: u16) -> u8 {
        self.mapper.borrow_mut().access_main_bus(addr, 0, false)
    }

    pub fn write_main_bus(&mut self, addr: u16, value: u8) {
        self.mapper.borrow_mut().access_main_bus(addr, value, true);
    }

    pub fn read_ppu_bus(&mut self, addr: u16) -> u8 {
        self.mapper.borrow_mut().access_ppu_bus(addr, 0, false)
    }

    pub fn write_ppu_bus(&mut self, addr: u16, value: u8) {
        self.mapper.borrow_mut().access_ppu_bus(addr, value, true);
    }
}
