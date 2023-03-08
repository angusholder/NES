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

use std::collections::VecDeque;
use std::error::Error;
use std::io::Write;
use std::panic::catch_unwind;
use std::path::Path;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::{Duration, Instant};
use sdl2::messagebox::{ButtonData, MessageBoxButtonFlag, MessageBoxFlag, show_message_box};
use sdl2::surface::Surface;
use crate::mapper::Mapper;
use crate::nes::NES;

pub const SCREEN_WIDTH: u32 = 256;
pub const SCREEN_HEIGHT: u32 = 240;
pub const SCREEN_PIXELS: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let result = catch_unwind(main_loop);
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            // An explicit error returned
            display_error_dialog("Unexpected error", &e.to_string());
        }
        Err(e) => {
            // The NES code panicked - probably an instruction/system not implemented yet, or a bug
            let mut err_msg: String = "Unknown error".to_string();
            if let Some(msg) = e.downcast_ref::<String>() {
                err_msg = msg.clone();
            } else if let Some(msg) = e.downcast_ref::<&str>() {
                err_msg = msg.to_string();
            }
            display_error_dialog("Unexpected runtime error", &err_msg);
        }
    }
}

fn main_loop() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut window = video_subsystem.window("NES Emulator", SCREEN_WIDTH*2, SCREEN_HEIGHT*2)
        .position_centered()
        .build()?;

    let mut display_buffer_paletted = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::Index8)?;
    display_buffer_paletted.set_palette(&load_nes_palette())?;
    let mut display_buffer_rgb = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::ARGB8888)?;

    let mut frame_stats = FrameStats::new();
    let mut event_pump = sdl_context.event_pump()?;
    let mut nes: Option<Box<NES>> = None;
    'running: loop {
        let start_time = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                Event::DropFile { filename, .. } => {
                    let trace_output: Option<Box<dyn Write>> = None; // Some(Box::new(std::fs::File::create("trace.txt").unwrap()));
                    match load_nes_system(&filename, trace_output) {
                        Ok(new_nes) => {
                            nes = Some(new_nes);
                        }
                        Err(e) => {
                            display_error_dialog("Failed to load the ROM", &e.to_string());
                        }
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

        window.set_title(&format!("NES Emulator - {:.2}ms", frame_stats.get_avg_frame_time_ms()))?;
        let frame_time = start_time.elapsed();
        frame_stats.add_reading(frame_time);
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60).saturating_sub(frame_time));
    }

    Ok(())
}

fn load_nes_system(
    filename: &String,
    trace_output: Option<Box<dyn Write>>,
) -> Result<Box<NES>, Box<dyn Error>> {
    let cart = cartridge::parse_rom(Path::new(&filename))?;
    let mapper = Mapper::new(cart)?;
    let mut nes = Box::new(NES::new(mapper, trace_output));
    nes.power_on();
    Ok(nes)
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

struct FrameStats {
    frame_count: u32,
    frame_times: VecDeque<Duration>,
}

const MAX_READINGS: usize = 60;

impl FrameStats {
    fn new() -> FrameStats {
        FrameStats {
            frame_count: 0,
            frame_times: VecDeque::with_capacity(MAX_READINGS),
        }
    }

    fn add_reading(&mut self, time: Duration) {
        self.frame_count += 1;
        self.frame_times.truncate(MAX_READINGS - 1);
        self.frame_times.push_back(time);
    }

    fn get_avg_frame_time_ms(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total_time: Duration = self.frame_times.iter().sum();
        let mean = total_time / self.frame_times.len() as u32;
        mean.as_micros() as f64 / 1000.0
    }
}
