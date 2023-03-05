#![allow(dead_code)]

mod cartridge;
mod nes;
mod ops;
#[cfg(test)]
mod test_instructions;
mod instructions;
mod ppu;
mod mapper;
mod disassemble;

use std::path::Path;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::Duration;
use sdl2::surface::Surface;
use crate::mapper::Mapper;
use crate::nes::NES;

pub const SCREEN_WIDTH: u32 = 256;
pub const SCREEN_HEIGHT: u32 = 240;
pub const SCREEN_PIXELS: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("NES Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut nes_display_surface = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::Index8).unwrap();
    nes_display_surface.set_palette(&load_nes_palette()).unwrap();

    let mut trace_file = std::fs::File::create("trace.txt").unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut nes: Option<NES> = None;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                Event::DropFile { filename, .. } => {
                    let cart = cartridge::parse_rom(Path::new(&filename)).unwrap();
                    let mapper = Mapper::new(cart).unwrap();
                    nes = Some(NES::new(mapper));
                    nes.as_mut().unwrap().power_on();
                }
                _ => {}
            }
        }

        if let Some(nes) = &mut nes {
            nes.simulate_frame(None);

            nes_display_surface.with_lock_mut(|pixels: &mut [u8]| {
                nes.ppu.output_display_buffer(pixels);
            })
        }
        let mut window_surf = window.surface(&event_pump).unwrap();
        nes_display_surface.blit(None, &mut window_surf, None).unwrap();
        window_surf.finish().unwrap();

        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn load_nes_palette() -> Palette {
    static PALETTE_LOOKUP: &[u8; 192] = include_bytes!("../ntscpalette_24bpp.pal");

    let mut colors = [Color::BLACK; 64];
    for i in 0..64 {
        let r = PALETTE_LOOKUP[i * 3 + 0];
        let g = PALETTE_LOOKUP[i * 3 + 1];
        let b = PALETTE_LOOKUP[i * 3 + 2];
        colors[i] = Color::RGB(r, g, b);
    }

    Palette::with_colors(&colors).unwrap()
}
