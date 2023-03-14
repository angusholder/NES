use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::{Cartridge, NametableMirroring};

mod mapper0;
mod mapper1;

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
            1 => Mapper::wrap(mapper1::MMC1Mapper::new(cart)),
            _ => {
                return Err(format!("Mapper #{} not supported yet", cart.mapper_num))
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
        self.mapper.borrow_mut().access_ppu_bus(mask_ppu_addr(addr), 0, false)
    }

    pub fn write_ppu_bus(&mut self, addr: u16, value: u8) {
        self.mapper.borrow_mut().access_ppu_bus(mask_ppu_addr(addr), value, true);
    }
}

/// The PPU address space is 14 bits, but the CPU address space is 16 bits.
/// "Valid addresses are $0000–$3FFF; higher addresses will be mirrored down" - https://www.nesdev.org/wiki/PPU_registers#Address_($2006)_%3E%3E_write_x2
#[inline(always)]
fn mask_ppu_addr(addr: u16) -> u16 {
    addr & 0x3FFF
}

/// See https://www.nesdev.org/wiki/Mirroring#Nametable_Mirroring
pub fn access_nametable(storage: &mut [u8; 0x800], mirroring: NametableMirroring, addr: u16) -> &mut u8 {
    let range = match mirroring {
        NametableMirroring::Horizontal => match addr {
            0x2000..=0x27FF => &mut storage[..0x400],
            0x2800..=0x2FFF => &mut storage[0x400..0x800],
            _ => panic!("Attempted to access nametable outside of range: {addr:04X}"),
        },
        NametableMirroring::Vertical => match addr {
            0x2000..=0x23FF | 0x2800..=0x2BFF => &mut storage[..0x400],
            0x2400..=0x27FF | 0x2C00..=0x2FFF => &mut storage[0x400..0x800],
            _ => panic!("Attempted to access nametable outside of range: {addr:04X}"),
        },
        NametableMirroring::SingleScreenLowerBank => match addr {
            0x2000..=0x2FFF => &mut storage[..0x400],
            _ => panic!("Attempted to access nametable outside of range: {addr:04X}"),
        },
        NametableMirroring::SingleScreenUpperBank => match addr {
            0x2000..=0x2FFF => &mut storage[0x400..0x800],
            _ => panic!("Attempted to access nametable outside of range: {addr:04X}"),
        },
    };
    &mut range[(addr & 0x3FF) as usize]
}
