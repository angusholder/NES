#![allow(dead_code)]

mod cartridge;
mod nes;
mod cpu_ops;
#[cfg(test)]
mod test_cpu;
mod cpu;
mod ppu;
mod mapper;
mod disassemble;
mod input;

use std::error::Error;
use std::io::Write;
use std::panic::catch_unwind;
use std::path::Path;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::Duration;
use sdl2::messagebox::{ButtonData, MessageBoxButtonFlag, MessageBoxFlag, show_message_box};
use sdl2::surface::Surface;
use crate::mapper::Mapper;
use crate::nes::NES;

pub const SCREEN_WIDTH: u32 = 256;
pub const SCREEN_HEIGHT: u32 = 240;
pub const SCREEN_PIXELS: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

fn main() {
    let result = catch_unwind(main_loop);
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            // An explicit error returned
            display_error_dialog("Unexpected error", &e.to_string());
        }
        Err(e) => {
            // The NES code panicked - probably an instruction/system not implemented yet, or a bug
            let placeholder = "Unknown error".to_string();
            let err_msg = e.downcast_ref::<String>().unwrap_or(&placeholder);
            display_error_dialog("Unexpected runtime error", &err_msg);
        }
    }
}

fn main_loop() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("NES Emulator", SCREEN_WIDTH*2, SCREEN_HEIGHT*2)
        .position_centered()
        .build()?;

    let mut display_buffer_paletted = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::Index8)?;
    display_buffer_paletted.set_palette(&load_nes_palette())?;
    let mut display_buffer_rgb = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::ARGB8888)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut nes: Option<NES> = None;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                Event::DropFile { filename, .. } => {
                    let trace_output: Option<Box<dyn Write>> = None; // Some(File::create("trace.txt").unwrap());
                    let result = load_nes_system(&mut nes, &filename, trace_output);
                    if let Err(e) = result {
                        display_error_dialog("Failed to load the ROM", &e.to_string());
                    }
                }
                _ => {}
            }
        }

        display_buffer_rgb.fill_rect(None, Color::BLACK)?;
        if let Some(nes) = &mut nes {
            nes.input.update_key_state(&event_pump);

            nes.simulate_frame();

            nes.ppu.output_display_buffer(display_buffer_paletted.without_lock_mut().unwrap());
            display_buffer_paletted.blit(None, &mut display_buffer_rgb, None).unwrap();
        }
        let mut window_surf = window.surface(&event_pump)?;
        display_buffer_rgb.blit_scaled(None, &mut window_surf, None)?;
        window_surf.finish()?;

        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

fn load_nes_system(
    nes_out: &mut Option<NES>,
    filename: &String,
    trace_output: Option<Box<dyn Write>>,
) -> Result<(), Box<dyn Error>> {
    let cart = cartridge::parse_rom(Path::new(&filename))?;
    let mapper = Mapper::new(cart)?;
    let mut nes = NES::new(mapper, trace_output);
    nes.power_on();
    *nes_out = Some(nes);
    Ok(())
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

fn display_error_dialog(title: &str, message: &str) {
    show_message_box(
        MessageBoxFlag::ERROR,
        &[ButtonData { text: "Close", button_id: 0, flags: MessageBoxButtonFlag::NOTHING }],
        title,
        message,
        None, None,
    ).unwrap();
}
