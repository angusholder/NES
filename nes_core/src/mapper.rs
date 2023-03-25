use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::{Cartridge, NametableMirroring};
use crate::nes::Signals;

mod nrom;
mod mmc1;
mod uxrom;
mod cnrom;
mod mmc2;
mod mmc3;

/// The mapper covers two address spaces - the CPU memory map, and the PPU memory map.
/// The CPU memory map is 16-bit, and the PPU memory map is 14-bit.
///
/// There's only one method for each address space, and the `write` parameter tells us whether we're
/// reading or writing (so we don't have to duplicate the address logic between reads and writes).
trait RawMapper {
    fn write_main_bus(&mut self, addr: u16, value: u8);
    fn read_main_bus(&mut self, addr: u16) -> u8;

    fn access_ppu_bus(&mut self, addr: u16, value: u8, write: bool) -> u8;

    fn on_cycle_scanline(&mut self) {}
}

#[derive(Clone)]
pub struct Mapper {
    mapper: Rc<RefCell<dyn RawMapper>>,
    pub signals: Rc<Signals>,
}

impl Mapper {
    pub fn new(cart: Cartridge) -> Result<Mapper, String> {
        let signals = Signals::new();
        Ok(match cart.mapper_num {
            0 => Mapper::wrap(nrom::NRomMapper::new(cart), signals),
            1 => Mapper::wrap(mmc1::MMC1Mapper::new(cart), signals),
            2 => Mapper::wrap(uxrom::UxRomMapper::new(cart), signals),
            3 => Mapper::wrap(cnrom::CNRomMapper::new(cart), signals),
            4 => Mapper::wrap(mmc3::MMC3Mapper::new(cart, signals.clone()), signals),
            9 => Mapper::wrap(mmc2::MMC2Mapper::new(cart), signals),
            _ => {
                return Err(format!("Mapper #{} not supported yet", cart.mapper_num))
            }
        })
    }

    fn wrap<M: RawMapper + 'static>(raw_mapper: M, signals: Rc<Signals>) -> Mapper {
        Mapper {
            mapper: Rc::new(RefCell::new(raw_mapper)),
            signals: Signals::new(),
        }
    }

    pub fn read_main_bus(&mut self, addr: u16) -> u8 {
        self.mapper.borrow_mut().read_main_bus(addr)
    }

    pub fn write_main_bus(&mut self, addr: u16, value: u8) {
        self.mapper.borrow_mut().write_main_bus(addr, value);
    }

    pub fn read_ppu_bus(&mut self, addr: u16) -> u8 {
        self.mapper.borrow_mut().access_ppu_bus(mask_ppu_addr(addr), 0, false)
    }

    pub fn write_ppu_bus(&mut self, addr: u16, value: u8) {
        self.mapper.borrow_mut().access_ppu_bus(mask_ppu_addr(addr), value, true);
    }

    pub fn on_cycle_scanline(&mut self) {
        self.mapper.borrow_mut().on_cycle_scanline();
    }
}

/// The PPU address space is 14 bits, but the CPU address space is 16 bits.
/// "Valid addresses are $0000–$3FFF; higher addresses will be mirrored down" - https://www.nesdev.org/wiki/PPU_registers#Address_($2006)_%3E%3E_write_x2
#[inline(always)]
fn mask_ppu_addr(addr: u16) -> u16 {
    addr & 0x3FFF
}

/// See https://www.nesdev.org/wiki/Mirroring#Nametable_Mirroring
pub fn access_nametable(storage: &mut [u8; 0x800], mirroring: NametableMirroring, addr: u16, value: u8, write: bool) -> u8 {
    let sub_addr = (addr & 0x3FF) as usize;
    let range = match mirroring {
        NametableMirroring::Horizontal => match addr {
            0x2000..=0x27FF => &mut storage[..0x400][sub_addr],
            0x2800..=0x2FFF => &mut storage[0x400..0x800][sub_addr],
            _ => unreachable!(),
        },
        NametableMirroring::Vertical => match addr {
            0x2000..=0x23FF | 0x2800..=0x2BFF => &mut storage[..0x400][sub_addr],
            0x2400..=0x27FF | 0x2C00..=0x2FFF => &mut storage[0x400..0x800][sub_addr],
            _ => unreachable!(),
        },
        NametableMirroring::SingleScreenLowerBank => match addr {
            0x2000..=0x2FFF => &mut storage[..0x400][sub_addr],
            _ => unreachable!(),
        },
        NametableMirroring::SingleScreenUpperBank => match addr {
            0x2000..=0x2FFF => &mut storage[0x400..0x800][sub_addr],
            _ => unreachable!(),
        },
    };
    if write {
        *range = value
    }
    *range
}

#[inline(never)]
#[track_caller]
#[cold]
pub fn out_of_bounds_read(context: &str, addr: u16) -> u8 {
    log::warn!("Attempted to read {context} out of bounds at {addr:04X}");

    return 0;
}

#[inline(never)]
#[track_caller]
#[cold]
pub fn out_of_bounds_write(context: &str, addr: u16, value: u8) {
    log::warn!("Attempted to write {context} out of bounds at {addr:04X} with {value} (0x{value:02X})");
}

#[inline(never)]
#[track_caller]
#[cold]
pub fn out_of_bounds_access(context: &str, addr: u16, value: u8, write: bool) -> u8 {
    if write {
        out_of_bounds_write(context, addr, value);
        return 0;
    } else {
        out_of_bounds_read(context, addr)
    }
}
