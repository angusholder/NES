use crate::cartridge::NametableMirroring;

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
        use self::NtAddr::*;
        let mut nt = NameTables {
            storage: [0; 0x1000],
            base_addrs: [Addr000, Addr000, Addr000, Addr000],
        };
        nt.update_mirroring(mirroring);
        nt
    }

    pub fn update_mirroring(&mut self, mirroring: NametableMirroring) {
        use self::NtAddr::*;

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
