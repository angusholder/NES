use crate::mapper::Mapper;
use crate::nes::{NES};
use crate::{SCREEN_PIXELS};

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

    /*
    During rendering:

    yyy NN YYYYY XXXXX
    ||| || ||||| +++++-- coarse X scroll
    ||| || +++++-------- coarse Y scroll
    ||| ++-------------- nametable select
    +++----------------- fine Y scroll
     */
    v_addr: u16,
    t_addr: u16,
    // https://www.nesdev.org/wiki/PPU_scrolling
    fine_x: u8,
    write_toggle_w: bool,
    data_bus_latch: u8,

    oam_addr: u8,
    oam: [u8; NUM_SPRITES * 4],
    cur_line_sprites: [SpriteRowSlice; 8],
    sprite_0_hit: bool,

    palettes: [u8; 2 * 4 * 4],
    mapper: Mapper,

    vblank_started: bool,

    /// Filled with values 0-63, which are indices into "ntscpalette_24bpp.pal".
    /// This is the in-progress frame that is being drawn.
    cur_display_buffer: [u8; SCREEN_PIXELS],
    /// Filled with values 0-63, which are indices into "ntscpalette_24bpp.pal".
    /// This is the finished frame, ready to be displayed.
    finished_display_buffer: [u8; SCREEN_PIXELS],
    frame_num: u64,

    dot: u32, // 0-340
    scanline: u32, // 0-261
    attribute_byte: u8,
    tiles_lo: u16,
    tiles_hi: u16,

    next_nametable_byte: u8,
    next_attribute_byte: u8,
    next_tile_lo: u8,
    next_tile_hi: u8,
}

/// The 0th element in this array is not used.
type Palette = [u8; 4];

impl PPU {
    pub fn new(mapper: Mapper) -> PPU {
        PPU {
            control: PPUControl::from_bits(0),
            mask: PPUMask::from_bits(0),

            v_addr: 0,
            t_addr: 0,
            fine_x: 0,
            write_toggle_w: false,
            data_bus_latch: 0,

            oam_addr: 0,
            oam: [0; NUM_SPRITES * 4],
            cur_line_sprites: [SpriteRowSlice::hidden(); 8],
            sprite_0_hit: false,

            palettes: [0; 2 * 4 * 4],
            mapper,

            vblank_started: true,

            cur_display_buffer: [0; 256 * 240],
            finished_display_buffer: [0; 256 * 240],
            frame_num: 0,

            dot: 0,
            scanline: 0,
            attribute_byte: 0,
            tiles_lo: 0,
            tiles_hi: 0,
            next_nametable_byte: 0,
            next_attribute_byte: 0,
            next_tile_lo: 0,
            next_tile_hi: 0,
        }
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.show_background || self.mask.show_sprites
    }

    fn write_mem(&mut self, addr: u16, val: u8) {
        if addr >= 0x3F00 && addr < 0x4000 {
            self.palettes[mask_palette_addr(addr)] = val;
        } else {
            self.mapper.write_ppu_bus(addr, val);
        }
    }

    fn read_mem(&mut self, addr: u16) -> u8 {
        if addr >= 0x3F00 && addr < 0x4000 {
            self.palettes[mask_palette_addr(addr)]
        } else {
            self.mapper.read_ppu_bus(addr)
        }
    }

    fn flip_frame(&mut self) {
        self.finished_display_buffer.copy_from_slice(&self.cur_display_buffer)
    }

    pub fn output_display_buffer(&self, output: &mut [u8]) {
        output.copy_from_slice(&self.finished_display_buffer);
    }
}

fn mask_palette_addr(addr: u16) -> usize {
    if addr == 0x3F10 {
        0
    } else {
        (addr & 0x1F) as usize
    }
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

impl SpriteSize {
    fn height(self) -> u32 {
        match self {
            SpriteSize::Size8x8 => 8,
            SpriteSize::Size8x16 => 16,
        }
    }
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

fn mask_register_addr(addr: u16) -> u16 { addr & 0x2007 }

pub fn ppu_read_register(nes: &mut NES, addr: u16) -> u8 {
    let ppu = &mut nes.ppu;
    match mask_register_addr(addr) {
        PPUSTATUS => {
            ppu.write_toggle_w = false;

            let mut status = 0u8;
            // TODO: Sprite overflow not implemented

            if ppu.vblank_started {
                status |= 0b1000_0000;
                ppu.vblank_started = false;
            }
            if ppu.sprite_0_hit {
                status |= 0b0100_0000;
            }

            // PPU open bus. Returns stale PPU bus contents
            status |= ppu.data_bus_latch & 0b0001_1111;

            // "Reading any readable port (PPUSTATUS, OAMDATA, or PPUDATA) also fills the latch with the bits read" - https://www.nesdev.org/wiki/PPU_registers#Ports
            ppu.data_bus_latch = status;
            status
        }
        OAMDATA => {
            let res = ppu.oam[ppu.oam_addr as usize];

            // "Reading any readable port (PPUSTATUS, OAMDATA, or PPUDATA) also fills the latch with the bits read" - https://www.nesdev.org/wiki/PPU_registers#Ports
            ppu.data_bus_latch = res;

            res
        }
        PPUDATA => {
            let res = ppu.read_mem(ppu.v_addr);
            ppu.v_addr += ppu.control.vram_increment as u16;

            // "Reading any readable port (PPUSTATUS, OAMDATA, or PPUDATA) also fills the latch with the bits read" - https://www.nesdev.org/wiki/PPU_registers#Ports
            ppu.data_bus_latch = res;

            res
        }
        PPUCTRL |
        PPUMASK |
        OAMADDR |
        PPUSCROLL |
        PPUADDR => {
            // Reading a nominally "write-only" register returns the latch's current value, as do the unused bits of PPUSTATUS. - https://www.nesdev.org/wiki/PPU_registers#Ports
            ppu.data_bus_latch
        }
        _ => unreachable!(),
    }
}

pub fn ppu_write_register(nes: &mut NES, addr: u16, val: u8) {
    let ppu = &mut nes.ppu;

    // "Writing any value to any PPU port, even to the nominally read-only PPUSTATUS, will fill this latch" - https://www.nesdev.org/wiki/PPU_registers#Ports
    ppu.data_bus_latch = val;

    match mask_register_addr(addr) {
        PPUCTRL => {
            ppu.control = PPUControl::from_bits(val);
            ppu.t_addr |= 0b11_00000_00000 & ppu.control.base_nametable_addr;
        }
        PPUMASK => {
            ppu.mask = PPUMask::from_bits(val);
        }
        PPUSTATUS => {
            // Do nothing, the only effect of writing PPUSTATUS is that of filling data_bus_latch.
        }
        OAMADDR => {
            ppu.oam_addr = val;
        }
        OAMDATA => {
            ppu.oam[ppu.oam_addr as usize] = val;
            ppu.oam_addr = ppu.oam_addr.wrapping_add(1);
        }
        PPUSCROLL => {
            if !ppu.write_toggle_w {
                ppu.fine_x = val & 0b111;
                ppu.t_addr = (ppu.t_addr & !0x1F) | (val >> 3) as u16;
            } else {
                let val = val as u16;
                ppu.t_addr = (ppu.t_addr & !0b0001100_00011111) | (val & 0b11111000 << 2) | (val & 0b111 << 12);
            }
            ppu.write_toggle_w = !ppu.write_toggle_w;
        }
        PPUADDR => {
            if !ppu.write_toggle_w {
                // Write upper byte first
                ppu.v_addr = (ppu.v_addr & 0x00FF) | ((val as u16) << 8);
            } else {
                // Then lower byte
                ppu.v_addr = (ppu.v_addr & 0xFF00) | (val as u16);
            }
            ppu.write_toggle_w = !ppu.write_toggle_w;
        }
        PPUDATA => {
            ppu.write_mem(ppu.v_addr, val);
            ppu.v_addr += ppu.control.vram_increment as u16;
        }
        _ => unreachable!(),
    }
}

/// https://www.nesdev.org/wiki/PPU_registers#OAM_DMA_($4014)_%3E_write
pub fn do_oam_dma(nes: &mut NES, source_upper_addr: u8) {
    let oam_addr = nes.ppu.oam_addr as usize;
    for i in 0..256 {
        nes.ppu.oam[oam_addr.wrapping_add(i) as usize] = nes.read8(((source_upper_addr as u16) << 8) + i as u16);
        nes.tick(); // PPU write cycle
    }
    nes.tick(); // 1 wait-state while waiting for writes to complete
    if nes.get_cycles() % 2 == 1 {
        nes.tick(); // 1 more cycle if odd
    }
}

const FIRST_SCANLINE: u32 = 0;
const LAST_SCANLINE: u32 = 261;
const DOTS_PER_SCANLINE: u32 = 341;

pub fn ppu_step(nes: &mut NES) {
    match nes.ppu.scanline {
        // Visible scanlines
        0..=239 => {
            ppu_step_scanline(&mut nes.ppu);
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
                nes.ppu.sprite_0_hit = false;
            }
            ppu_step_scanline(&mut nes.ppu);
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

fn ppu_step_scanline(ppu: &mut PPU) {
    let dot = ppu.dot;
    let scanline = ppu.scanline;

    // See the cycles here https://www.nesdev.org/wiki/PPU_rendering#Visible_scanlines_(0-239)
    match dot {
        1..=256 | 321..=336 => {
            render_pixel(ppu);
            // Background fetches - https://www.nesdev.org/wiki/File:Ppu.svg
            match dot % 8 {
                1  => {
                    let tile_addr = 0x2000 | (ppu.v_addr & 0x0FFF);
                    ppu.next_nametable_byte = ppu.read_mem(tile_addr);
                }
                3 => {
                    let v = ppu.v_addr;
                    let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                    ppu.next_attribute_byte = ppu.read_mem(attr_addr);
                }
                5 => {
                    ppu.next_tile_lo = ppu.read_mem(get_tile_address(ppu.control.background_pattern_table, ppu.next_nametable_byte, scanline % 8, false))
                }
                7 => {
                    ppu.next_tile_hi = ppu.read_mem(get_tile_address(ppu.control.background_pattern_table, ppu.next_nametable_byte, scanline % 8, true));
                }
                0 => {
                    ppu.attribute_byte = ppu.next_attribute_byte;
                    ppu.tiles_lo = (ppu.tiles_lo & 0x00FF) | (ppu.next_tile_lo.reverse_bits() as u16) << 8;
                    ppu.tiles_hi = (ppu.tiles_hi & 0x00FF) | (ppu.next_tile_hi.reverse_bits() as u16) << 8;
                    if ppu.rendering_enabled() {
                        scroll_next_x(ppu);
                    }
                }
                _ => {}
            }
        }
        // Sprite-loading interval
        257..=320 => {
            if ppu.rendering_enabled() {
                ppu.oam_addr = 0;

                // We totally ignore the real cycles here, & just do all evaluation in one cycle.
                // I don't think there's an observable difference between this and the real thing.
                // https://www.nesdev.org/wiki/PPU_sprite_evaluation
                if dot == 257 {
                    ppu.cur_line_sprites = evaluate_sprites_for_line(ppu, scanline + 1);
                }
            }
        }
        _ => {}
    }

    if dot == 256 && ppu.rendering_enabled() {
        scroll_next_y(ppu);
    }
    if dot == 257 && ppu.rendering_enabled() {
        update_x_from_temp(ppu);
    }
    // Pre-render scanline, copy vertical bits from t to v
    if scanline == 261 && matches!(dot, 280..=304) && ppu.rendering_enabled() {
        update_y_from_temp(ppu);
    }
}

fn update_x_from_temp(ppu: &mut PPU) {
// If rendering is enabled, the PPU copies all bits related to horizontal position from t to v - https://www.nesdev.org/wiki/PPU_scrolling
    let horizontal_mask = 0b100_00011111;
    ppu.v_addr = (ppu.v_addr & !horizontal_mask) | (ppu.t_addr & horizontal_mask);
}

fn update_y_from_temp(ppu: &mut PPU) {
    let vertical_mask = 0b1111011_11100000;
    ppu.v_addr = (ppu.v_addr & !vertical_mask) | (ppu.t_addr & vertical_mask);
}

/// https://www.nesdev.org/wiki/PPU_scrolling#Between_dot_328_of_a_scanline,_and_256_of_the_next_scanline
/// https://www.nesdev.org/wiki/PPU_scrolling#Coarse_X_increment
fn scroll_next_x(ppu: &mut PPU) {
    if ppu.v_addr & 0x001F == 31 { // Coarse X == 31
        ppu.v_addr = (ppu.v_addr & !0x001F) // coarse X = 0
            ^ 0x0400; // switch horizontal nametable
    } else {
        ppu.v_addr += 1;
    }
}

/// https://www.nesdev.org/wiki/PPU_scrolling#Y_increment
fn scroll_next_y(ppu: &mut PPU) {
    if ppu.v_addr & 0x7000 != 0x7000 { // fine Y < 7
        ppu.v_addr += 0x1000; // increment fine Y
    } else {
        ppu.v_addr &= !0x7000; // fine Y = 0
        let mut y = (ppu.v_addr & 0x03E0) >> 5;
        if y == 29 {
            y = 0; // coarse Y = 0
            ppu.v_addr ^= 0x0800; // switch vertical nametable
        } else if y == 31 {
            y = 0; // coarse Y = 0, nametable not switched
        } else {
            y += 1;
        }
        ppu.v_addr = (ppu.v_addr & !0x03E0) | (y << 5);
    }
}

fn render_pixel(ppu: &mut PPU) {
    let x = ppu.dot.wrapping_sub(2);
    if ppu.scanline < 240 && x < 256 {
        let mut bg_color_index = 0;
        if ppu.mask.show_background && (ppu.mask.show_background_left || x > 8) {
            let palette_index = read_attribute_byte(ppu.attribute_byte, ppu.dot, ppu.scanline);
            bg_color_index = (ppu.tiles_lo >> ppu.fine_x & 1) as u8 | ((ppu.tiles_hi >> ppu.fine_x & 1) << 1) as u8;
            if bg_color_index != 0 {
                bg_color_index |= palette_index << 2;
            }
        }

        let mut sprite_color_index: u8 = 0;
        let mut sprite_behind_bg: bool = true;
        let mut is_sprite_0 = false;
        if ppu.mask.show_sprites && (ppu.mask.show_sprites_left || x > 8) && ppu.scanline > 0 {
            let mut i = 0;
            while i < ppu.cur_line_sprites.len() {
                let sprite = &ppu.cur_line_sprites[i];
                let sx = sprite.x as u32;
                if sx <= x && x < sx + 8 {
                    let dx = x - sx;
                    sprite_color_index = (sprite.pattern2 >> (dx*2)) as u8 & 0b11;
                    if sprite_color_index != 0 {
                        sprite_color_index |= 0x10 | (sprite.palette_index << 2);
                    }
                    sprite_behind_bg = sprite.behind_bg;
                    is_sprite_0 = sprite.is_sprite_0;
                    break;
                }
                i += 1;
            }
        }

        // Choose a pixel based on priority
        let mut pixel_index = bg_color_index;
        if sprite_color_index != 0 { // Sprite pixel not blank
            if bg_color_index == 0 { // Background pixel is blank
                pixel_index = sprite_color_index;
            } else {
                if !sprite_behind_bg {
                    pixel_index = sprite_color_index;
                }

                // Sprite 0 hit ignores priority, it only requires that both sprite pixel and
                // bg pixel be non-transparent.
                if is_sprite_0 && x != 255 && !ppu.sprite_0_hit {
                    ppu.sprite_0_hit = true;
                }
            }

        }

        ppu.cur_display_buffer[(ppu.scanline * 256 + x) as usize] = ppu.palettes[pixel_index as usize];
    }

    // These shift registers need to shift even if we're not rendering pixels, so that cycles
    // 321-336 correctly prefetch the first two tiles for the next scanline
    ppu.tiles_lo >>= 1;
    ppu.tiles_hi >>= 1;
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

const NUM_SPRITES: usize = 64;

#[derive(Clone, Copy, Debug)]
struct SpriteRowSlice {
    x: u8,
    // two bits per pixel
    pattern2: u16,
    behind_bg: bool,
    palette_index: u8,
    is_sprite_0: bool,
}

impl SpriteRowSlice {
    fn hidden() -> SpriteRowSlice {
        SpriteRowSlice {
            x: 0xFF,
            pattern2: 0x0000,
            behind_bg: true,
            palette_index: 0,
            is_sprite_0: false,
        }
    }
}

const SPRITE_Y: usize = 0;
const SPRITE_TILE_INDEX: usize = 1;
const SPRITE_ATTRIBUTES: usize = 2;
const SPRITE_X: usize = 3;

const SPRITE_ATTR_BEHIND_BG: u8 = 0b0010_0000;
const SPRITE_ATTR_PALETTE: u8 = 0b0000_0011;
const SPRITE_ATTR_FLIP_H: u8 = 0b0100_0000;
const SPRITE_ATTR_FLIP_V: u8 = 0b1000_0000;

fn evaluate_sprites_for_line(ppu: &mut PPU, line: u32) -> [SpriteRowSlice; 8] {
    let mut sprites = [SpriteRowSlice::hidden(); 8];
    let mut dest_index = 0usize;
    let sprite_size = ppu.control.sprite_size;
    let sprite_height = sprite_size.height();
    for src_index in 0..NUM_SPRITES {
        let sprite_data: [u8; 4] = ppu.oam[src_index * 4 .. (src_index + 1) * 4].try_into().unwrap();
        let y = sprite_data[SPRITE_Y] as u32;
        let y_range = y..y + sprite_height;
        if !y_range.contains(&line) {
            continue;
        }

        let attrs = sprite_data[SPRITE_ATTRIBUTES];
        let tile_index = sprite_data[SPRITE_TILE_INDEX];
        let pattern2: u16 = match sprite_size {
            SpriteSize::Size8x8 => {
                let mut y_offset = line - y;
                if attrs & SPRITE_ATTR_FLIP_V != 0 {
                    y_offset = 7 - y_offset;
                }
                let pattern_addr = get_tile_address(ppu.control.sprite_pattern_table, tile_index, y_offset, false);
                let mut pat_lower = ppu.read_mem(pattern_addr);
                let mut pat_upper = ppu.read_mem(pattern_addr + 8);
                if attrs & SPRITE_ATTR_FLIP_H == 0 {
                    pat_lower = pat_lower.reverse_bits();
                    pat_upper = pat_upper.reverse_bits();
                }
                interleave_bits(pat_lower, pat_upper)
            }
            SpriteSize::Size8x16 => {
                // TODO: Implement 8x16 sprites
                0
            }
        };

        sprites[dest_index] = SpriteRowSlice {
            x: sprite_data[SPRITE_X],
            pattern2,
            behind_bg: (attrs & SPRITE_ATTR_BEHIND_BG) != 0,
            palette_index: (attrs & SPRITE_ATTR_PALETTE),
            is_sprite_0: src_index == 0,
        };
        dest_index += 1;
        if dest_index >= sprites.len() {
            break;
        }
    }
    sprites
}

fn interleave_bits_slow(x: u8, y: u8) -> u16 {
    let mut result = 0u16;
    for i in 0..8 {
        let bits = ((x >> i) & 1) as u16 | (((y >> i) & 1) as u16) << 1;
        result |= bits << (i * 2);
    }
    result
}

/// Interleaves bits like so:
/// interleave_bits(0b00, 0b11) == 0b1010
/// interleave_bits(0b11, 0b00) == 0b0101
fn interleave_bits(lower: u8, upper: u8) -> u16 {
    let x = lower as u64;
    let y = upper as u64;
    let res = ((x.wrapping_mul(0x0101010101010101) & 0x8040201008040201).wrapping_mul(0x0102040810204081) >> 49) & 0x5555 |
        ((y.wrapping_mul(0x0101010101010101) & 0x8040201008040201).wrapping_mul(0x0102040810204081) >> 48) & 0xAAAA;
    res as u16
}

#[test]
fn test_interleave_bits() {
    assert_eq!(interleave_bits(0b00, 0b11), 0b1010);
    assert_eq!(interleave_bits(0b11, 0b00), 0b0101);
    for x in 0..=255 {
        for y in 0..=255 {
            let slow_res = interleave_bits_slow(x, y);
            let fast_res = interleave_bits(x, y);
            assert_eq!(slow_res, fast_res);
        }
    }
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
