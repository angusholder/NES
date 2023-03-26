use std::rc::Rc;
use log::{info, warn};
use crate::cartridge::{Cartridge, NametableMirroring};
use crate::mapper;
use crate::mapper::RawMapper;
use crate::nes::Signals;

pub struct MMC3Mapper {
    chr_rom: Vec<u8>,
    prg_rom: Vec<u8>,
    mirroring: NametableMirroring,
    nametables: [u8; 0x800],
    prg_ram: Option<[u8; 8 * 1024]>,

    bank_reg: [u8; 8],
    // The next write to the bank data register will affect bank_reg[bank_reg_select]
    bank_reg_select: u8,
    prg_bank_mode: PRGBankMode,
    /// 0: two 2KB banks at $0000-$0FFF, four 1KB banks at $1000-$1FFF
    /// 1: two 2KB banks at $1000-$1FFF, four 1KB banks at $0000-$0FFF
    chr_a12_inversion: bool,

    irq_counter: u8,
    irq_counter_reload_value: u8,
    irq_counter_reload: bool,
    irq_enable: bool,

    prg_banks: [usize; 4],
    chr_banks: [usize; 8],
    signals: Rc<Signals>,
}

#[derive(Debug)]
enum PRGBankMode {
    /// $8000-$9FFF swappable, $C000-$DFFF fixed to second-last bank
    Swappable89 = 0,
    /// $C000-$DFFF swappable, $8000-$9FFF fixed to second-last bank
    SwappableCD = 1,
}

const PRG_BANK_SIZE: usize = 0x2000; // 8KB
const CHR_BANK_SIZE: usize = 0x400; // 1KB

impl MMC3Mapper {
    pub fn new(cart: Cartridge, signals: Rc<Signals>) -> MMC3Mapper {
        let prg_ram = match cart.prg_ram_size {
            8192 => Some([0; 8192]),
            0 => None,
            other => {
                warn!("Unexpected PRG RAM size {other}");
                None
            }
        };

        let mut mapper = MMC3Mapper {
            chr_rom: cart.chr_rom,
            prg_rom: cart.prg_rom,
            mirroring: cart.mirroring,
            nametables: [0; 0x800],
            prg_ram,

            bank_reg: [0; 8],
            bank_reg_select: 0,
            prg_bank_mode: PRGBankMode::Swappable89,
            chr_a12_inversion: false,

            irq_counter: 0,
            irq_counter_reload_value: 0,
            irq_counter_reload: false,
            irq_enable: false,

            prg_banks: [0; 4],
            chr_banks: [0; 8],
            signals,
        };

        // Initialize prg_banks and chr_banks.
        mapper.sync_mappings();

        mapper
    }

    fn sync_mappings(&mut self) {
        let old_chr_banks = self.chr_banks;
        self.chr_banks = [
            // 0x0000-0x07FF
            self.bank_reg[0] as usize * CHR_BANK_SIZE,
            (self.bank_reg[0]+1) as usize * CHR_BANK_SIZE,

            // 0x0800-0x0FFF
            self.bank_reg[1] as usize * CHR_BANK_SIZE,
            (self.bank_reg[1]+1) as usize * CHR_BANK_SIZE,

            // 0x1000-0x13FF
            self.bank_reg[2] as usize * CHR_BANK_SIZE,
            // 0x1400-0x17FF
            self.bank_reg[3] as usize * CHR_BANK_SIZE,
            // 0x1800-0x1BFF
            self.bank_reg[4] as usize * CHR_BANK_SIZE,
            // 0x1C00-0x1FFF
            self.bank_reg[5] as usize * CHR_BANK_SIZE,
        ];

        // if self.chr_a12_inversion {
        //     // Flip address bit A12
        //     let (lower, upper) = self.chr_banks.split_at_mut(4);
        //     lower.swap_with_slice(upper);
        // }

        if self.chr_banks != old_chr_banks {
            info!("CHR Banks: [{:05X}, {:05X}, {:05X}, {:05X}, {:05X}, {:05X}, {:05X}, {:05X}]",
                self.chr_banks[0], self.chr_banks[1], self.chr_banks[2], self.chr_banks[3],
                self.chr_banks[4], self.chr_banks[5], self.chr_banks[6], self.chr_banks[7]);
        }

        let old_prg_banks = self.prg_banks;
        self.prg_banks = match self.prg_bank_mode {
            PRGBankMode::Swappable89 => [
                self.bank_reg[6] as usize * PRG_BANK_SIZE, // R6
                self.bank_reg[7] as usize * PRG_BANK_SIZE, // R7
                self.prg_rom.len() - 2*PRG_BANK_SIZE, // 2nd last page
                self.prg_rom.len() - 1*PRG_BANK_SIZE, // Last page
            ],
            PRGBankMode::SwappableCD => [
                self.prg_rom.len() - 2*PRG_BANK_SIZE, // 2nd last page
                self.bank_reg[7] as usize * PRG_BANK_SIZE, // R7
                self.bank_reg[6] as usize * PRG_BANK_SIZE, // R6
                self.prg_rom.len() - 1*PRG_BANK_SIZE, // Last page
            ],
        };
        if old_prg_banks != self.prg_banks {
            info!("PRG Banks {:?}: [{:05X}, {:05X}, {:05X}, {:05X}]", self.prg_bank_mode,
                self.prg_banks[0], self.prg_banks[1], self.prg_banks[2], self.prg_banks[3]);
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xE001 {
            // Bank select
            0x8000 => {
                self.bank_reg_select = value & 0b111;
                info!("Selected R{}", self.bank_reg_select);
                self.chr_a12_inversion = value & 0x80 != 0;
                self.prg_bank_mode = if value & 0x40 == 0 { PRGBankMode::Swappable89 } else { PRGBankMode::SwappableCD };
                self.sync_mappings();
            }
            // Bank data
            0x8001 => {
                let sel = self.bank_reg_select as usize;
                self.bank_reg[sel] = value;
                if sel == 0 || sel == 1 {
                    // Odd-numbered banks can't be selected by the 2KB bank slots.
                    self.bank_reg[sel] &= !1;
                }
                if sel == 6 || sel == 7 {
                    // There's only 6 PRG ROM address lines
                    self.bank_reg[sel] &= 0b0011_1111;
                }
                info!("R{sel} = {}", self.bank_reg[sel]);
                self.sync_mappings();
            }
            // Mirroring
            0xA000 => {
                self.mirroring = match value & 1 {
                    0 => NametableMirroring::Vertical,
                    1 => NametableMirroring::Horizontal,
                    _ => unreachable!(),
                };
                info!("{:?} mirroring", self.mirroring);
            }
            // PRG RAM protect
            0xA001 => {
                // Not implemented
            }
            // IRQ latch
            0xC000 => {
                self.irq_counter_reload_value = value;
            }
            // IRQ reload
            0xC001 => {
                // Triggers the counter to load the reload value upon the next scanline cycle.
                self.irq_counter = 0;
            }
            // IRQ disable
            0xE000 => {
                self.irq_enable = false;
                // TODO: Do we need a separate variable for our IRQ vs other IRQs in the system?
                self.signals.acknowledge_irq();
            }
            // IRQ enable
            0xE001 => {
                self.irq_enable = true;
            }
            _ => unreachable!(),
        }
    }
}

impl RawMapper for MMC3Mapper {
    fn write_main_bus(&mut self, addr: u16, value: u8) {
        match addr {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = self.prg_ram.as_mut() {
                    prg_ram[(addr & 0x1FFF) as usize] = value;
                } else {
                    mapper::out_of_bounds_write("PRG RAM", addr, value);
                }
            }
            0x8000..=0xFFFF => {
                self.write_register(addr, value);
            }
            _ => {
                mapper::out_of_bounds_write("cart", addr, value)
            }
        }
    }

    fn read_main_bus(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = self.prg_ram.as_ref() {
                    prg_ram[(addr & 0x1FFF) as usize]
                } else {
                    mapper::out_of_bounds_read("PRG RAM", addr)
                }
            }
            0x8000..=0x9FFF => self.prg_rom[self.prg_banks[0] + (addr & 0x1FFF) as usize],
            0xA000..=0xBFFF => self.prg_rom[self.prg_banks[1] + (addr & 0x1FFF) as usize],
            0xC000..=0xDFFF => self.prg_rom[self.prg_banks[2] + (addr & 0x1FFF) as usize],
            0xE000..=0xFFFF => self.prg_rom[self.prg_banks[3] + (addr & 0x1FFF) as usize],
            // 0x8000..=0xFFFF => {
            //     // 4 banks of 0x2000/8KB each
            //     let bank_no = (addr >> 0x1FFFu32.count_ones() & 0b11) as usize;
            //     let low_addr = (addr & 0x1FFF) as usize;
            //     self.prg_rom[self.prg_banks[bank_no] + low_addr]
            // }
            _ => mapper::out_of_bounds_read("CPU memory space", addr)
        }
    }

    fn access_ppu_bus(&mut self, mut addr: u16, value: u8, write: bool) -> u8 {
        if self.chr_a12_inversion {
            // Flip address bit A12
            addr ^= 1 << 12;
        }
        match addr {
            0x0000..=0x1FFF if !write => {
                // 8 banks of 0x400/1KB each
                let bank_no = ((addr >> 0x3FFu32.count_ones()) & 0b111) as usize;
                let low_addr = (addr & 0x3FF) as usize;
                self.chr_rom[self.chr_banks[bank_no] + low_addr]
            }
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                mapper::access_nametable(&mut self.nametables, self.mirroring, addr & 0x2FFF, value, write)
            }
            _ => {
                mapper::out_of_bounds_access("PPU memory space", addr, value, write)
            }
        }
    }

    fn on_cycle_scanline(&mut self) {
        if self.irq_counter == 0 && self.irq_enable {
            self.signals.request_irq();
        }
        if self.irq_counter == 0 || self.irq_counter_reload {
            self.irq_counter = self.irq_counter_reload_value;
        } else {
            self.irq_counter -= 1;
        }
    }
}
