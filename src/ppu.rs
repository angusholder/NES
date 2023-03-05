use log::warn;
use crate::mapper::Mapper;
use crate::nes::{NES};

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

    vblank_started: bool,

    cur_display_buffer: [u8; 256 * 240],
    finished_display_buffer: [u8; 256 * 240],
    frame_num: u64,

    dot: u32, // 0-340
    scanline: u32, // 0-261
    attribute_byte: u8,
    low_tile_byte: u8,
    high_tile_byte: u8,

    next_nametable_byte: u8,
    next_attribute_byte: u8,
    next_low_tile_byte: u8,
    next_high_tile_byte: u8,
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

            vblank_started: true,

            cur_display_buffer: [0; 256 * 240],
            finished_display_buffer: [0; 256 * 240],
            frame_num: 0,

            dot: 0,
            scanline: 0,
            attribute_byte: 0,
            low_tile_byte: 0,
            high_tile_byte: 0,
            next_nametable_byte: 0,
            next_attribute_byte: 0,
            next_low_tile_byte: 0,
            next_high_tile_byte: 0,
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

    fn flip_frame(&mut self) {
        self.finished_display_buffer.copy_from_slice(&self.cur_display_buffer)
    }

    pub fn output_display_buffer(&self, output: &mut [u8], pitch: usize) {
        assert_eq!(pitch, 256 * 4);
        assert_eq!(output.len(), self.finished_display_buffer.len() * 4);
        for (i, pixel) in self.finished_display_buffer.iter().enumerate() {
            let color = self.palette_to_rgb[*pixel as usize];
            let offset = i * 4;
            output[offset..offset+4].copy_from_slice(&color.to_le_bytes());
        }
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
    vram_increment: u16,
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
            let mut status = 0u8;

            if nes.ppu.vblank_started {
                status |= 0b1000_0000;
                nes.ppu.vblank_started = false;
            }

            status
        }
        OAMADDR => unimplemented!(),
        OAMDATA => unimplemented!(),
        PPUSCROLL => unimplemented!(),
        PPUADDR => unimplemented!(),
        PPUDATA => {
            let res = nes.ppu.read_mem(nes.ppu.ppu_addr);
            nes.ppu.ppu_addr += nes.ppu.control.vram_increment as u16;
            res
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
        PPUSCROLL => {
            warn!("Ignoring PPUSCROLL = {:#x}", val);
        }
        PPUADDR => {
            nes.ppu.ppu_addr = nes.ppu.ppu_addr << 8 | val as u16;
        }
        PPUDATA => {
            nes.ppu.write_mem(nes.ppu.ppu_addr, val);
            nes.ppu.ppu_addr += nes.ppu.control.vram_increment as u16;
        }
        _ => unreachable!(),
    }
}

const FIRST_SCANLINE: u32 = 0;
const LAST_SCANLINE: u32 = 261;
const DOTS_PER_SCANLINE: u32 = 341;

pub fn ppu_step(nes: &mut NES) {
    match nes.ppu.scanline {
        // Visible scanlines
        0..=239 => {
            ppu_step_scanline(nes);
        }
        240 => {
            if nes.ppu.dot == 0 {
                // A full frame has been rendered, make it visible
                nes.ppu.flip_frame();
            }
        }
        241 => {
            if nes.ppu.dot == 1 {
                nes.ppu.vblank_started = true;
                if nes.ppu.control.enable_nmi {
                    nes.trigger_nmi = true;
                }
            }
        }
        // Pre-render line - a dummy scanline to fill the shift registers ready for line 0
        261 => {
            if nes.ppu.dot == 1 {
                nes.ppu.vblank_started = false;
            }
            ppu_step_scanline(nes);
        }
        _ => {}
    }

    let ppu = &mut nes.ppu;
    ppu.dot += 1;
    if ppu.dot >= DOTS_PER_SCANLINE {
        ppu.dot = 0;
        ppu.scanline += 1;
        if ppu.scanline > LAST_SCANLINE {
            ppu.scanline = FIRST_SCANLINE;
            ppu.frame_num += 1;
        }
    }
}

fn ppu_step_scanline(nes: &mut NES) {
    let ppu: &mut PPU = &mut nes.ppu;

    let dot = ppu.dot;
    let scanline = ppu.scanline;
    let y_offset = scanline; // TODO: Apply PPUSCROLL

    // See the cycles here https://www.nesdev.org/wiki/PPU_rendering#Visible_scanlines_(0-239)
    match dot {
        1..=256 | 321..=336 => {
            render_pixel(ppu);
            // Background fetches - https://www.nesdev.org/wiki/File:Ppu.svg
            match dot % 8 {
                0 => {
                    ppu.attribute_byte = ppu.next_attribute_byte;
                    ppu.low_tile_byte = ppu.next_low_tile_byte;
                    ppu.high_tile_byte = ppu.next_high_tile_byte;
                }
                1  => {
                    ppu.next_nametable_byte = ppu.read_mem(pixel_to_nametable_addr(dot, scanline));
                }
                3 => {
                    ppu.next_attribute_byte = ppu.read_mem(pixel_to_attribute_addr(dot, scanline));
                }
                5 => {
                    ppu.next_low_tile_byte = ppu.read_mem(get_tile_address(ppu.control.background_pattern_table, ppu.next_nametable_byte, y_offset % 8, false))
                }
                7 => {
                    ppu.next_high_tile_byte = ppu.read_mem(get_tile_address(ppu.control.background_pattern_table, ppu.next_nametable_byte, y_offset % 8, true));
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn render_pixel(ppu: &mut PPU) {
    let x = ppu.dot.wrapping_sub(2);
    if ppu.scanline < 240 && x < 256 {
        let color_index = ppu.low_tile_byte & 1 | (ppu.high_tile_byte & 1) << 1;
        ppu.low_tile_byte >>= 1;
        ppu.high_tile_byte >>= 1;
        let palette_index = read_attribute_byte(ppu.attribute_byte, ppu.dot, ppu.scanline);
        let color = if color_index == 0 {
            ppu.universal_bg_color
        } else {
            ppu.bg_palettes[palette_index as usize][color_index as usize]
        };
        ppu.cur_display_buffer[(ppu.scanline * 256 + x) as usize] = color;
    }
}

fn pixel_to_nametable_addr(x: u32, y: u32) -> u16 {
    let mut tile_x = (x / 8) as u16;
    let mut tile_y = (y / 8) as u16;
    let base_addr: u16 = match (tile_x, tile_y) {
        (0..=31, 0..=29) => 0x2000,
        (32..=63, 0..=29) => 0x2400,
        (0..=31, 30..=59) => 0x2800,
        (32..=63, 30..=59) => 0x2C00,
        _ => unreachable!(),
    };
    if tile_y > 29 {
        tile_y -= 30;
    }
    if tile_x > 31 {
        tile_x -= 32;
    }
    tile_y * 32 + tile_x + base_addr
}

fn pixel_to_attribute_addr(x: u32, y: u32) -> u16 {
    let mut tile_x = (x / 8) as u16;
    let mut tile_y = (y / 8) as u16;
    let base_addr: u16 = match (tile_x, tile_y) {
        (0..=31, 0..=29) => 0x23C0,
        (32..=63, 0..=29) => 0x27C0,
        (0..=31, 30..=59) => 0x2BC0,
        (32..=63, 30..=59) => 0x2FC0,
        _ => unreachable!(),
    };
    if tile_y > 29 {
        tile_y -= 30;
    }
    if tile_x > 31 {
        tile_x -= 32;
    }
    (tile_y / 4) * 8 + (tile_x / 4) + base_addr
}

fn get_tile_address(base_addr: u16, tile_no: u8, y_offset: u32, high: bool) -> u16 {
    assert!(y_offset < 8);
    let mut tile_addr = base_addr + (tile_no as u16) * 16 + (y_offset as u16);
    if high {
        tile_addr += 8;
    }
    tile_addr
}

fn read_attribute_byte(attribute: u8, x: u32, y: u32) -> u8 {
    let mut shift: u32 = 0;
    if x & 16 != 0 {
        shift += 2;
    }
    if y & 16 != 0 {
        shift += 4;
    }
    attribute >> shift & 0b11
}

#[test]
fn test_pixel_to_nametable_addr() {
    for y in 0..30 {
        println!("{} = {:04X}", y, pixel_to_nametable_addr(0, y * 8));
    }
    assert_eq!(pixel_to_nametable_addr(0, 0), 0x2000);
    assert_eq!(pixel_to_nametable_addr(1, 0), 0x2000);
    assert_eq!(pixel_to_nametable_addr(8, 0), 0x2001);
    assert_eq!(pixel_to_nametable_addr(248, 0), 0x2000 + 31);
    assert_eq!(pixel_to_nametable_addr(255, 0), 0x2000 + 31);
    assert_eq!(pixel_to_nametable_addr(0, 1), 0x2000);
    assert_eq!(pixel_to_nametable_addr(0, 8), 0x2000 + 32);
    assert_eq!(pixel_to_nametable_addr(0, 232), 0x23A0);
    assert_eq!(pixel_to_nametable_addr(0, 239), 0x23A0);
    assert_eq!(pixel_to_nametable_addr(255, 239), 0x23BF);
    assert_eq!(pixel_to_nametable_addr(248, 239), 0x23BF);

    assert_eq!(pixel_to_nametable_addr(256, 0), 0x2400);

    assert_eq!(pixel_to_nametable_addr(0, 240), 0x2800);

    assert_eq!(pixel_to_nametable_addr(256, 240), 0x2C00);
}

#[test]
fn test_pixel_to_attribute_table_addr() {
    assert_eq!(pixel_to_attribute_addr(0, 0), 0x23C0);
    assert_eq!(pixel_to_attribute_addr(256-32, 0), 0x23C7);
    assert_eq!(pixel_to_attribute_addr(255, 0), 0x23C7);
    assert_eq!(pixel_to_attribute_addr(0, 32), 0x23C8);
    assert_eq!(pixel_to_attribute_addr(0, 64), 0x23D0);
}

#[test]
fn test_tile_address() {
    assert_eq!(get_tile_address(0x0000, 0, 0, false), 0x0000);
    assert_eq!(get_tile_address(0x0000, 0, 0, true ), 0x0008);
    assert_eq!(get_tile_address(0x0000, 0, 1, false), 0x0001);
    assert_eq!(get_tile_address(0x0000, 0, 1, true ), 0x0009);
    assert_eq!(get_tile_address(0x0000, 0, 7, false), 0x0007);
    assert_eq!(get_tile_address(0x0000, 0, 7, true ), 0x000F);

    assert_eq!(get_tile_address(0x1000, 0, 0, false), 0x1000);
    assert_eq!(get_tile_address(0x1000, 0, 0, true ), 0x1008);
    assert_eq!(get_tile_address(0x1000, 0, 1, false), 0x1001);
    assert_eq!(get_tile_address(0x1000, 0, 1, true ), 0x1009);
    assert_eq!(get_tile_address(0x1000, 0, 7, false), 0x1007);
    assert_eq!(get_tile_address(0x1000, 0, 7, true ), 0x100F);

    assert_eq!(get_tile_address(0x0000, 1, 0, false), 0x0010);
}
