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
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::Duration;
use crate::mapper::Mapper;
use crate::nes::NES;

pub const SCREEN_WIDTH: u32 = 256;
pub const SCREEN_HEIGHT: u32 = 240;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("NES Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut tex: sdl2::render::Texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGBA32, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut trace_file = std::fs::File::create("trace.txt").unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    let mut nes: Option<NES> = None;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
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
            nes.simulate_frame(Some(&mut trace_file));
            tex.with_lock(None, |pixels, pitch| {
                nes.ppu.output_display_buffer(pixels, pitch);
            }).unwrap();
        }
        canvas.copy(&tex, None, None).unwrap();

        canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
