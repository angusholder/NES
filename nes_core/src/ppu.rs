use std::rc::Rc;
use crate::mapper::Mapper;
use crate::nes::{InterruptSource, NES, Signals};

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
    cur_line_num_sprites: usize, // Between 0 and 8
    sprite_0_hit: bool,

    palettes: [u8; 2 * 4 * 4],
    mapper: Rc<Mapper>,

    vblank_started: bool,
    signals: Rc<Signals>,

    /// Filled with values 0-63, which are indices into "ntscpalette_24bpp.pal".
    /// This is the in-progress frame that is being drawn.
    cur_display_buffer: Box<[u8; SCREEN_PIXELS]>,
    /// Filled with values 0-63, which are indices into "ntscpalette_24bpp.pal".
    /// This is the finished frame, ready to be displayed.
    finished_display_buffer: Box<[u8; SCREEN_PIXELS]>,
    frame_num: u64,

    dot: u32, // 0-340
    scanline: u32, // 0-261
    tiles_palette_lo: u16,
    tiles_palette_hi: u16,
    tiles_lo: u16,
    tiles_hi: u16,
}

impl PPU {
    pub fn new(mapper: Rc<Mapper>, signals: Rc<Signals>) -> PPU {
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
            cur_line_num_sprites: 0,
            sprite_0_hit: false,

            palettes: [0; 2 * 4 * 4],
            mapper,

            vblank_started: true,
            signals,

            cur_display_buffer: Box::new([0; 256 * 240]),
            finished_display_buffer: Box::new([0; 256 * 240]),
            frame_num: 0,

            dot: 0,
            scanline: 0,
            tiles_palette_lo: 0,
            tiles_palette_hi: 0,
            tiles_lo: 0,
            tiles_hi: 0,
        }
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.show_background_or_sprites
    }

    fn write_mem(&mut self, addr: u16, val: u8) {
        // PPU address bus is 14 bits, mask out the upper bits
        match addr & 0x3FFF {
            0x0000..=0x1FFF => {
                self.mapper.write_pattern_table(addr, val)
            }
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                self.mapper.write_nametable(addr, val);
            }
            0x3F00..=0x3FFF => {
                self.palettes[mask_palette_addr(addr)] = val & 0b11_1111;
            }
            _ => unreachable!(),
        }
    }

    fn read_mem(&mut self, addr: u16) -> u8 {
        // PPU address bus is 14 bits, mask out the upper bits
        match addr & 0x3FFF {
            0x0000..=0x1FFF => {
                self.mapper.read_pattern_table(addr)
            }
            0x2000..=0x2FFF | 0x3000..=0x3EFF => {
                self.mapper.read_nametable(addr)
            }
            0x3F00..=0x3FFF => {
                self.palettes[mask_palette_addr(addr)] & self.mask.grayscale_mask
            }
            _ => unreachable!(),
        }
    }

    fn flip_frame(&mut self) {
        self.finished_display_buffer.copy_from_slice(&self.cur_display_buffer[..])
    }

    pub fn output_display_buffer_rgb(&self, output: &mut [Color; SCREEN_PIXELS]) {
        for (i, palette_index) in self.finished_display_buffer.iter().enumerate() {
            // wrap into range of 64 colors
            output[i] = get_output_color(*palette_index);
        }
    }

    pub fn output_display_buffer_u32_argb(&self, output: &mut [u32; SCREEN_PIXELS]) {
        for (i, palette_index) in self.finished_display_buffer.iter().enumerate() {
            // wrap into range of 64 colors
            let Color { r, g, b } = get_output_color(*palette_index);
            output[i] = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
        }
    }

    pub fn output_display_buffer_indexed(&self, output: &mut[u8; SCREEN_PIXELS]) {
        output.copy_from_slice(&self.finished_display_buffer[..])
    }
}

pub const SCREEN_WIDTH: u32 = 256;
pub const SCREEN_HEIGHT: u32 = 240;
pub const SCREEN_PIXELS: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

#[derive(Debug, Default, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

static PALETTE_LOOKUP: &[u8; 192] = include_bytes!("../../nestopia_rgb.pal");

fn get_output_color(palette_index: u8) -> Color {
    let r = PALETTE_LOOKUP[palette_index as usize * 3 + 0];
    let g = PALETTE_LOOKUP[palette_index as usize * 3 + 1];
    let b = PALETTE_LOOKUP[palette_index as usize * 3 + 2];

    Color { r, g, b }
}

pub fn get_palette_colors() -> [Color; 64] {
    let mut res = [Color::default(); 64];
    for i in 0..64 {
        let r = PALETTE_LOOKUP[i * 3 + 0];
        let g = PALETTE_LOOKUP[i * 3 + 1];
        let b = PALETTE_LOOKUP[i * 3 + 2];
        res[i] = Color { r, g, b };
    }
    res
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
    fn from_bits(bits: u8) -> PPUControl {
        PPUControl {
            enable_nmi: bits & 0b1000_0000 != 0,
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct PPUMask {
    /// Bit 0 controls a greyscale mode, which causes the palette to use only the colors from the
    /// grey column: $00, $10, $20, $30. This is implemented as a bitwise AND with $30 on any value
    /// read from PPU $3F00-$3FFF, both on the display and through PPUDATA. Writes to the palette
    /// through PPUDATA are not affected. Also note that black colours like $0F will be replaced
    /// by a non-black grey $00.
    grayscale_mask: u8,
    show_background_left: bool,
    show_sprites_left: bool,
    show_background: bool,
    show_sprites: bool,
    show_background_or_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl PPUMask {
    fn from_bits(val: u8) -> PPUMask {
         PPUMask {
            grayscale_mask: if val & 0b0000_0001 != 0 { 0x30 } else { 0xFF },
            show_background_left: val & 0b0000_0010 != 0,
            show_sprites_left: val & 0b0000_0100 != 0,
            show_background: val & 0b0000_1000 != 0,
            show_sprites: val & 0b0001_0000 != 0,
            show_background_or_sprites: val & 0b0001_1000 != 0,
            emphasize_red: val & 0b0010_0000 != 0,
            emphasize_green: val & 0b0100_0000 != 0,
            emphasize_blue: val & 0b1000_0000 != 0,
         }
    }
}

impl PPU {
    pub fn read_register(&mut self, addr: u16) -> u8 {
        let ppu = self;

        match addr & 0x2007 {
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
                let addr = ppu.v_addr;
                let res = ppu.read_mem(addr);
                ppu.v_addr += ppu.control.vram_increment;

                let previous_read = ppu.data_bus_latch;
                // "Reading any readable port (PPUSTATUS, OAMDATA, or PPUDATA) also fills the latch with the bits read" - https://www.nesdev.org/wiki/PPU_registers#Ports
                ppu.data_bus_latch = res;

                if addr >= 0x3EFF {
                    // The palette memory responds immediately
                    res
                } else {
                    // The rest of PPU memory has an intermediate buffer, so we return the data
                    // that was read from memory on the previous PPUDATA read
                    previous_read
                }
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

    pub fn write_register(&mut self, addr: u16, val: u8) {
        let ppu = self;

        // "Writing any value to any PPU port, even to the nominally read-only PPUSTATUS, will fill this latch" - https://www.nesdev.org/wiki/PPU_registers#Ports
        ppu.data_bus_latch = val;

        match addr & 0x2007 {
            PPUCTRL => {
                ppu.control = PPUControl::from_bits(val);
                let nt_mask = 0b11_00000_00000;
                ppu.t_addr = (ppu.t_addr & !nt_mask) | (ppu.control.base_nametable_addr & nt_mask);
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
                    ppu.t_addr = (ppu.t_addr & 0b111_11_11111_00000) | (val >> 3) as u16;
                } else {
                    let val = val as u16;
                    ppu.t_addr = (ppu.t_addr & 0b000_11_00000_11111) | ((val & 0b11111000) << 2) | ((val & 0b111) << 12);
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
                ppu.v_addr += ppu.control.vram_increment;
            }
            _ => unreachable!(),
        }
    }
}

/// https://www.nesdev.org/wiki/PPU_registers#OAM_DMA_($4014)_%3E_write
pub fn do_oam_dma(nes: &mut NES, source_upper_addr: u8) {
    let oam_addr = nes.ppu.oam_addr;
    for i in 0..=255 {
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

impl PPU {
    pub fn step_cycle(&mut self) {
        let ppu = self;

        match ppu.scanline {
            // Visible scanlines
            0..=239 => {
                do_scanline_rendering(ppu);
            }
            241 => {
                if ppu.dot == 1 {
                    // A full frame has been rendered, make it visible
                    ppu.flip_frame();
                    ppu.vblank_started = true;
                    if ppu.control.enable_nmi {
                        ppu.signals.request_interrupt(InterruptSource::VBLANK_NMI);
                    }
                }
            }
            // Pre-render line - a dummy scanline to fill the shift registers ready for line 0
            261 => {
                if ppu.dot == 1 {
                    ppu.vblank_started = false;
                    ppu.sprite_0_hit = false;
                }
                do_scanline_rendering(ppu);

                // Copy vertical bits from t to v
                if matches!(ppu.dot, 280..=304) && ppu.rendering_enabled() {
                    update_y_from_temp(ppu);
                }
            }
            _ => {}
        }

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
}

fn do_scanline_rendering(ppu: &mut PPU) {
    let dot = ppu.dot;

    // See the cycles here https://www.nesdev.org/wiki/PPU_rendering#Visible_scanlines_(0-239)
    match dot {
        0..=256 | 321..=336 => {
            // Background fetches - https://www.nesdev.org/wiki/File:Ppu.svg
            if dot > 0 && dot % 8 == 0 {
                // Cycles 1 & 2
                let tile_addr = 0x2000 | (ppu.v_addr & 0x0FFF);
                let tile_index: u8 = ppu.mapper.read_nametable(tile_addr);

                // Cycles 3 & 4
                let palette_index: u8 = read_next_palette_index(ppu);

                // Cycles 5 & 6
                let fine_y: u16 = ppu.v_addr >> 12 & 0b111;
                let pattern_addr = ppu.control.background_pattern_table + (tile_index as u16) * 16 + fine_y;
                let next_tile_lo: u8 = ppu.mapper.read_pattern_table(pattern_addr);

                // Cycles 7 & 0
                let next_tile_hi: u8 = ppu.mapper.read_pattern_table(pattern_addr + 8);

                ppu.tiles_palette_lo = (ppu.tiles_palette_lo & 0xFF00) | if palette_index & 1 != 0 { 0x00FF } else { 0x0000 };
                ppu.tiles_palette_hi = (ppu.tiles_palette_hi & 0xFF00) | if palette_index & 2 != 0 { 0x00FF } else { 0x0000 };
                ppu.tiles_lo = (ppu.tiles_lo & 0xFF00) | next_tile_lo as u16;
                ppu.tiles_hi = (ppu.tiles_hi & 0xFF00) | next_tile_hi as u16;

                if ppu.rendering_enabled() {
                    scroll_next_x(ppu);
                }
            }

            let x = ppu.dot;
            if ppu.scanline < 240 && x < 256 {
                render_pixel(ppu, x);
            }

            // These shift registers need to shift even if we're not rendering pixels, so that cycles
            // 321-336 correctly prefetch the first two tiles for the next scanline
            if dot > 0 {
                ppu.tiles_lo <<= 1;
                ppu.tiles_hi <<= 1;
                ppu.tiles_palette_lo <<= 1;
                ppu.tiles_palette_hi <<= 1;
            }
        }
        // Sprite-loading interval
        257..=320 => {
            if ppu.rendering_enabled() {
                ppu.oam_addr = 0;

                // We totally ignore the real cycles here, & just do all evaluation in one cycle.
                // I don't think there's an observable difference between this and the real thing.
                // https://www.nesdev.org/wiki/PPU_sprite_evaluation
                if dot == 257 && ppu.scanline != 261 { // Sprite evaluation doesn't happen on the pre-render scanline
                    // We're evaluating the sprites for the next line, but since sprite evaluation
                    // doesn't occur on the pre-render scanline, no sprites can appear on line 0,
                    // so the sprite Y values are offset by one. This means at the end of line 0 we
                    // evaluate sprites for Y=0 then display them on line 1.
                    let line = ppu.scanline;
                    evaluate_sprites_for_line(ppu, line);
                }
            }
        }
        _ => {}
    }

    // TODO: This isn't totally accurate, but do we need to be more accurate? https://www.nesdev.org/wiki/MMC3#IRQ_Specifics
    if dot == 260 {
        ppu.mapper.on_cycle_scanline();
    }
    if dot == 256 && ppu.rendering_enabled() {
        scroll_next_y(ppu);
    }
    if dot == 257 && ppu.rendering_enabled() {
        update_x_from_temp(ppu);
    }
}

fn update_x_from_temp(ppu: &mut PPU) {
// If rendering is enabled, the PPU copies all bits related to horizontal position from t to v - https://www.nesdev.org/wiki/PPU_scrolling
    let horizontal_mask = 0b000_01_00000_11111;
    ppu.v_addr = (ppu.v_addr & !horizontal_mask) | (ppu.t_addr & horizontal_mask);
}

fn update_y_from_temp(ppu: &mut PPU) {
    let vertical_mask = 0b111_10_11111_00000;
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

fn render_pixel(ppu: &mut PPU, x: u32) {
    let mut bg_color_index = 0;
    if ppu.mask.show_background && (ppu.mask.show_background_left || x > 8) {
        let x_shift: u8 = 15 - ppu.fine_x;
        bg_color_index = (ppu.tiles_lo >> x_shift & 1) as u8 | ((ppu.tiles_hi >> x_shift & 1) << 1) as u8;
        if bg_color_index != 0 {
            let palette_index = (ppu.tiles_palette_lo >> x_shift & 1) as u8 | ((ppu.tiles_palette_hi >> x_shift & 1) << 1) as u8;
            bg_color_index |= palette_index << 2;
        }
    }

    let mut pixel_index = bg_color_index;

    if ppu.mask.show_sprites && (ppu.mask.show_sprites_left || x > 8) && ppu.scanline > 0 {
        let mut i = 0;
        while i < ppu.cur_line_num_sprites {
            let sprite = &ppu.cur_line_sprites[i];
            if sprite.start_x as u32 > x {
                i += 1;
                continue;
            }
            if x >= sprite.end_x as u32 {
                i += 1;
                continue;
            }

            // We've checked that start_x is less than or equal to x, so dx is always positive.
            let dx = x - sprite.start_x as u32;
            let sprite_color_index = (sprite.pattern2 >> (dx*2)) as u8 & 0b11;
            if sprite_color_index != 0 { // Sprite pixel not blank
                // Background pixel is blank, or sprite takes priority
                if bg_color_index == 0 || sprite.above_bg {
                    pixel_index = sprite_color_index | sprite.palette_base_addr;
                }

                // Sprite 0 hit occurs when both sprite and background are non-transparent (regardless of priority).
                if sprite.is_sprite_0 && bg_color_index != 0 && x != 255 {
                    ppu.sprite_0_hit = true;
                }

                break;
            }
            i += 1;
        }
    }

    ppu.cur_display_buffer[(ppu.scanline * 256 + x) as usize] = ppu.palettes[pixel_index as usize] & ppu.mask.grayscale_mask;
}

fn read_next_palette_index(ppu: &mut PPU) -> u8 {
    let v = ppu.v_addr as u32;
    let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
    let attr = ppu.mapper.read_nametable(attr_addr as u16);

    let mut shift: u32 = 0;
    if (v >> 1 & 1) != 0 { // the 2nd bit of coarse X (the 16s digit of X)
        shift += 2;
    }
    if (v >> 6 & 1) != 0 { // the 2nd bit of coarse Y (the 16s digit of Y)
        shift += 4;
    }

    attr >> shift & 0b11
}

const NUM_SPRITES: usize = 64;

#[derive(Clone, Copy, Debug)]
struct SpriteRowSlice {
    start_x: u8,
    // Larger than start_x in case of overflow
    end_x: u16,
    // two bits per pixel
    pattern2: u16,
    above_bg: bool,
    palette_base_addr: u8,
    is_sprite_0: bool,
}

impl SpriteRowSlice {
    fn hidden() -> SpriteRowSlice {
        SpriteRowSlice {
            start_x: 0xFF,
            end_x: 0xFF,
            pattern2: 0x0000,
            above_bg: false,
            palette_base_addr: 0,
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

fn evaluate_sprites_for_line(ppu: &mut PPU, line: u32) {
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
        let pattern_addr: u16;
        match sprite_size {
            SpriteSize::Size8x8 => {
                let mut y_offset = line - y;
                if attrs & SPRITE_ATTR_FLIP_V != 0 {
                    y_offset = 7 - y_offset;
                }
                pattern_addr = ppu.control.sprite_pattern_table + (tile_index as u16) * 16 + (y_offset as u16);
            }
            SpriteSize::Size8x16 => {
                let mut y_offset = line - y;
                let pattern_table = if tile_index & 1 == 1 { 0x1000 } else { 0x0000 };
                let mut tile_index = tile_index & !1;
                if attrs & SPRITE_ATTR_FLIP_V != 0 {
                    if y_offset >= 8 {
                        y_offset -= 8;
                    } else {
                        tile_index += 1;
                    }
                    y_offset = 7 - y_offset;
                } else {
                    if y_offset >= 8 {
                        tile_index += 1;
                        y_offset -= 8;
                    }
                }
                pattern_addr = pattern_table + (tile_index as u16) * 16 + (y_offset as u16);
            }
        };

        let mut pat_lower = ppu.mapper.read_pattern_table(pattern_addr);
        let mut pat_upper = ppu.mapper.read_pattern_table(pattern_addr + 8);
        if attrs & SPRITE_ATTR_FLIP_H == 0 {
            pat_lower = pat_lower.reverse_bits();
            pat_upper = pat_upper.reverse_bits();
        }
        let pattern2 = interleave_bits(pat_lower, pat_upper);

        let start_x: u8 = sprite_data[SPRITE_X];
        ppu.cur_line_sprites[dest_index] = SpriteRowSlice {
            start_x,
            end_x: start_x as u16 + 8,
            pattern2,
            above_bg: (attrs & SPRITE_ATTR_BEHIND_BG) == 0,
            palette_base_addr: 0x10 | ((attrs & SPRITE_ATTR_PALETTE) << 2),
            is_sprite_0: src_index == 0,
        };
        dest_index += 1;
        if dest_index >= ppu.cur_line_sprites.len() {
            break;
        }
    }
    ppu.cur_line_num_sprites = dest_index;
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
    fn interleave_bits_slow(x: u8, y: u8) -> u16 {
        let mut result = 0u16;
        for i in 0..8 {
            let bits = ((x >> i) & 1) as u16 | (((y >> i) & 1) as u16) << 1;
            result |= bits << (i * 2);
        }
        result
    }

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
