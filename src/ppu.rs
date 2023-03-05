use crate::mapper::Mapper;
use crate::nes::NES;

const PPUCTRL: u16 = 0x2000;
const PPUMASK: u16 = 0x2001;
const PPUSTATUS: u16 = 0x2002;
const OAMADDR: u16 = 0x2003;
const OAMDATA: u16 = 0x2004;
const PPUSCROLL: u16 = 0x2005;
const PPUADDR: u16 = 0x2006;
const PPUDATA: u16 = 0x2007;

pub struct PPU {
    control: PPUControl,
    mask: PPUMask,

    ppu_addr: u16,

    universal_bg_color: u8,
    bg_palettes: [Palette; 4],
    sprite_palettes: [Palette; 4],
    palette_to_rgb: Palette2RGB,
    mapper: Mapper,
}

type Palette2RGB = [u32; 64];
/// The 0th element in this array is not used.
type Palette = [u8; 4];

impl PPU {
    pub fn new(mapper: Mapper) -> PPU {
        PPU {
            control: PPUControl::from_bits(0),
            mask: PPUMask::from_bits(0),

            ppu_addr: 0,

            universal_bg_color: 0,
            bg_palettes: [Palette::default(); 4],
            sprite_palettes: [Palette::default(); 4],
            palette_to_rgb: get_palette_to_rgb(),
            mapper,
        }
    }

    fn write_mem(&mut self, addr: u16, val: u8) {
        self.access_mem::<true>(addr, val);
    }

    fn read_mem(&mut self, addr: u16) -> u8 {
        self.access_mem::<false>(addr, 0)
    }

    fn access_mem<const WRITE: bool>(&mut self, mut addr: u16, val: u8) -> u8 {
        addr &= 0x3FFF; // "Valid addresses are $0000–$3FFF; higher addresses will be mirrored down" - https://www.nesdev.org/wiki/PPU_registers#Address_($2006)_%3E%3E_write_x2

        if addr >= 0x3F00 && addr < 0x4000 {
            self.access_palette(&addr, val, WRITE)
        } else {
            if WRITE {
                self.mapper.write_ppu_bus(addr, val);
                0
            } else {
                self.mapper.read_ppu_bus(addr)
            }
        }
    }

    fn access_palette(&mut self, addr: &u16, val: u8, write: bool) -> u8 {
        let ptr: &mut u8 = match addr & 0x3F1F {
            0x3F00 | 0x3F10 => {
                &mut self.universal_bg_color
            }
            0x3F01..=0x3F0F => {
                &mut self.bg_palettes[(addr >> 2 & 0b11) as usize][(addr & 0b11) as usize]
            }
            0x3F11..=0x3F1F => {
                &mut self.sprite_palettes[(addr >> 2 & 0b11) as usize][(addr & 0b11) as usize]
            }
            _ => unreachable!(),
        };

        if write {
            *ptr = val;
        }
        *ptr
    }
}

fn get_palette_to_rgb() -> Palette2RGB {
    // From https://www.nesdev.org/wiki/PPU_palettes - 2C03 and 2C05
    static LOOKUP: [u16; 64] = [
        0o333,0o014,0o006,0o326,0o403,0o503,0o510,0o420,0o320,0o120,0o031,0o040,0o022,0o000,0o000,0o000,
        0o555,0o036,0o027,0o407,0o507,0o704,0o700,0o630,0o430,0o140,0o040,0o053,0o044,0o000,0o000,0o000,
        0o777,0o357,0o447,0o637,0o707,0o737,0o740,0o750,0o660,0o360,0o070,0o276,0o077,0o000,0o000,0o000,
        0o777,0o567,0o657,0o757,0o747,0o755,0o764,0o772,0o773,0o572,0o473,0o276,0o467,0o000,0o000,0o000,
    ];
    let mut result: Palette2RGB = [0; 64];
    for i in 0..64 {
        let color = LOOKUP[i] as u32;
        let r = (color >> 6) & 0b111;
        let g = (color >> 3) & 0b111;
        let b = (color >> 0) & 0b111;
        result[i] = (r << 16) | (g << 8) | b;
    }
    result
}

#[derive(Debug, Clone, Copy)]
struct PPUControl {
    enable_nmi: bool,
    slave_mode: bool,
    sprite_size: SpriteSize,
    background_pattern_table: u16,
    sprite_pattern_table: u16,
    // add 1 (going across), or add 32 (going down)
    vram_increment: u8,
    base_nametable_addr: u16,
}

#[derive(Debug, Clone, Copy)]
enum SpriteSize {
    Size8x8,
    Size8x16,
}

impl PPUControl {
    pub fn from_bits(bits: u8) -> PPUControl {
        PPUControl {
            enable_nmi: bits & 0b1000_0000 != 0,
            slave_mode: bits & 0b0100_0000 != 0,
            sprite_size: if bits & 0b0010_0000 != 0 {
                SpriteSize::Size8x16
            } else {
                SpriteSize::Size8x8
            },
            background_pattern_table: if bits & 0b0001_0000 != 0 { 0x1000 } else { 0x0000 },
            sprite_pattern_table: if bits & 0b0000_1000 != 0 { 0x1000 } else { 0x0000 },
            vram_increment: if bits & 0b0000_0100 != 0 { 32 } else { 1 },
            base_nametable_addr: match bits & 0b0000_0011 {
                0b00 => 0x2000,
                0b01 => 0x2400,
                0b10 => 0x2800,
                0b11 => 0x2C00,
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PPUMask {
    grayscale: bool,
    show_background_left: bool,
    show_sprites_left: bool,
    show_background: bool,
    show_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl PPUMask {
    fn from_bits(val: u8) -> PPUMask {
         PPUMask {
            grayscale: val & 0b0000_0001 != 0,
            show_background_left: val & 0b0000_0010 != 0,
            show_sprites_left: val & 0b0000_0100 != 0,
            show_background: val & 0b0000_1000 != 0,
            show_sprites: val & 0b0001_0000 != 0,
            emphasize_red: val & 0b0010_0000 != 0,
            emphasize_green: val & 0b0100_0000 != 0,
            emphasize_blue: val & 0b1000_0000 != 0,
         }
    }
}

fn mask_ppu_addr(addr: u16) -> u16 { addr & 0x2007 }

pub fn ppu_read_register(nes: &mut NES, addr: u16) -> u8 {
    match mask_ppu_addr(addr) {
        PPUCTRL => unimplemented!(),
        PPUMASK => unimplemented!(),
        PPUSTATUS => {
            // TODO: Reset PPUADDR
            // VBlank set
            0x80
        }
        OAMADDR => unimplemented!(),
        OAMDATA => unimplemented!(),
        PPUSCROLL => unimplemented!(),
        PPUADDR => unimplemented!(),
        PPUDATA => {
            nes.ppu.read_mem(nes.ppu.ppu_addr)
        }
        _ => unreachable!(),
    }
}

pub fn ppu_write_register(nes: &mut NES, addr: u16, val: u8) {
    match mask_ppu_addr(addr) {
        PPUCTRL => {
            nes.ppu.control = PPUControl::from_bits(val);
            println!("PPUCTRL = {:#?}", nes.ppu.control)
        }
        PPUMASK => {
            nes.ppu.mask = PPUMask::from_bits(val);
            println!("PPUMASK = {:#?}", nes.ppu.mask)
        }
        PPUSTATUS => unimplemented!(),
        OAMADDR => unimplemented!(),
        OAMDATA => unimplemented!(),
        PPUSCROLL => unimplemented!(),
        PPUADDR => {
            nes.ppu.ppu_addr = nes.ppu.ppu_addr << 8 | val as u16;
        }
        PPUDATA => {
            nes.ppu.write_mem(nes.ppu.ppu_addr, val);
        }
        _ => unreachable!(),
    }
}

