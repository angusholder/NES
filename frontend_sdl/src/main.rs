use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::panic::catch_unwind;
use std::path::Path;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use std::time::{Duration, Instant};
use log::info;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired};
use sdl2::controller::{Button, GameController};
use sdl2::EventPump;
use sdl2::messagebox::{ButtonData, MessageBoxButtonFlag, MessageBoxFlag, show_message_box};
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::Window;
use nes_core::apu::{AudioChannels, SampleBuffer};
use nes_core::cartridge;
use nes_core::input::JoypadButtons;
use nes_core::mapper::Mapper;
use nes_core::nes::NES;
use nes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, self};

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
    let controller_subsystem = sdl_context.game_controller().unwrap();
    info!("Video driver: {}", video_subsystem.current_video_driver());

    let window: Window = video_subsystem.window("NES Emulator", SCREEN_WIDTH*3, SCREEN_HEIGHT*3)
        .position_centered()
        .build()?;
    info!("Window {:?}", window.display_mode()?);

    let mut canvas: WindowCanvas = window.into_canvas()
        .accelerated()
        .present_vsync()
        .build()?;
    info!("Renderer {:?}", canvas.info());

    let texture_creator: TextureCreator<_> = canvas.texture_creator();

    let mut display_texture: Texture = texture_creator.create_texture_streaming(PixelFormatEnum::ARGB8888, SCREEN_WIDTH, SCREEN_HEIGHT)?;

    let mut display_buffer_paletted = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::Index8)?;
    let colors = ppu::get_palette_colors().map(|c| Color { r: c.r, g: c.g, b: c.b, a: 255 });
    let palette = Palette::with_colors(&colors)?;
    display_buffer_paletted.set_palette(&palette)?;
    let mut display_buffer_rgb = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::ARGB8888)?;

    let mut audio_device: AudioDevice<NesAudioCallback> = create_audio_device(&sdl_context);
    info!("Got audio device: {:?}", audio_device.spec());

    let keymap: Keymap = get_key_map();

    let mut controller_mappings = &include_bytes!("../gamecontrollerdb.txt")[..];
    controller_subsystem.load_mappings_from_read(&mut controller_mappings).unwrap();
    let mut game_controller: Option<GameController> = None;

    let mut frame_stats = FrameStats::new();
    let mut event_pump = sdl_context.event_pump()?;
    let mut nes: Option<Box<NES>> = None;
    let mut paused = false;
    'running: loop {
        let start_time = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    paused = !paused;
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    let Some(nes) = nes.as_mut() else { continue; };
                    match keycode {
                        Keycode::Num1 => nes.apu.toggle_channel(AudioChannels::SQUARE1),
                        Keycode::Num2 => nes.apu.toggle_channel(AudioChannels::SQUARE2),
                        Keycode::Num3 => nes.apu.toggle_channel(AudioChannels::TRIANGLE),
                        Keycode::Num4 => nes.apu.toggle_channel(AudioChannels::NOISE),
                        Keycode::Num5 => nes.apu.toggle_channel(AudioChannels::DMC),
                        _ => {}
                    }
                }
                Event::ControllerDeviceAdded { which: joystick_index, .. } => {
                    let controller = controller_subsystem.open(joystick_index).unwrap();
                    if controller.instance_id() == 0 {
                        info!("P1 game controller plugged in: {}, attached={}, joystick_index={joystick_index}, instance_id={}", controller.name(), controller.attached(), controller.instance_id());
                        game_controller = Some(controller);
                    } else {
                        info!("Other game controller plugged in, ignoring ({}, attached={}, joystick_index={joystick_index}, instance_id={})", controller.name(), controller.attached(), controller.instance_id());
                    }
                }
                Event::ControllerDeviceRemoved { which: instance_id, .. } => {
                    info!("Controller device {instance_id} removed");
                    if game_controller.as_ref().map(|c| c.instance_id()) == Some(instance_id) {
                        game_controller = None;
                        info!("No game controller present now");
                    }
                }
                Event::DropFile { filename, .. } => {
                    match load_nes_system(&filename) {
                        Ok(mut new_nes) => {
                            let mut sample_buffer = audio_device.lock().get_output_buffer();
                            sample_buffer.clear();
                            new_nes.apu.attach_output_device(sample_buffer);
                            audio_device.resume();
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

        if !paused {
            if let Some(nes) = &mut nes {
                nes.input.update_key_state(get_pressed_buttons(&event_pump, &keymap, game_controller.as_ref()));

                nes.simulate_frame();

                render_nes_to_surface(&mut display_buffer_paletted, nes);
                display_buffer_paletted.blit(None, &mut display_buffer_rgb, None).unwrap();
            }
        }
        display_texture.update(None, display_buffer_rgb.without_lock().unwrap(), display_buffer_rgb.pitch() as usize)?;

        canvas.clear();
        canvas.copy(&display_texture, None, None)?;
        canvas.present();

        let pause_text = if paused { " - PAUSED" } else { "" };
        canvas.window_mut().set_title(&format!("NES Emulator - {:.2}ms{}", frame_stats.get_avg_frame_time_ms(), pause_text))?;
        let frame_time = start_time.elapsed();
        frame_stats.add_reading(frame_time);
    }

    Ok(())
}

fn render_nes_to_surface(display_buffer_rgb: &mut Surface, nes: &mut NES) {
    nes.ppu.output_display_buffer_indexed(display_buffer_rgb.without_lock_mut().unwrap().try_into().unwrap());
}

fn load_nes_system(
    filename: &String,
) -> Result<Box<NES>, Box<dyn Error>> {
    let cart = cartridge::parse_rom(Path::new(&filename))?;
    let mapper = Mapper::new(cart)?;
    let mut nes = Box::new(NES::new(mapper));
    nes.power_on();
    Ok(nes)
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

type Keymap = HashMap<Scancode, JoypadButtons>;

fn get_key_map() -> Keymap {
    let mut map = HashMap::new();
    map.insert(Scancode::Z, JoypadButtons::A);
    map.insert(Scancode::X, JoypadButtons::B);
    map.insert(Scancode::A, JoypadButtons::SELECT);
    map.insert(Scancode::S, JoypadButtons::START);
    map.insert(Scancode::Return, JoypadButtons::START);
    map.insert(Scancode::Up, JoypadButtons::UP);
    map.insert(Scancode::Down, JoypadButtons::DOWN);
    map.insert(Scancode::Left, JoypadButtons::LEFT);
    map.insert(Scancode::Right, JoypadButtons::RIGHT);
    map
}

pub fn get_pressed_buttons(event_pump: &EventPump, keymap: &Keymap, controller: Option<&GameController>) -> JoypadButtons {
    let mut pressed = JoypadButtons::empty();
    for (scan_code, button) in keymap.iter() {
        if event_pump.keyboard_state().is_scancode_pressed(*scan_code) {
            pressed.insert(*button);
        }
    }
    if let Some(con) = controller {
        if con.button(Button::A) { pressed |= JoypadButtons::A; }
        if con.button(Button::B) { pressed |= JoypadButtons::B; }
        if con.button(Button::DPadUp) { pressed |= JoypadButtons::UP; }
        if con.button(Button::DPadDown) { pressed |= JoypadButtons::DOWN; }
        if con.button(Button::DPadLeft) { pressed |= JoypadButtons::LEFT; }
        if con.button(Button::DPadRight) { pressed |= JoypadButtons::RIGHT; }
        if con.button(Button::Start) { pressed |= JoypadButtons::START; }
    }
    pressed
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
        while self.frame_times.len() >= MAX_READINGS {
            self.frame_times.pop_front();
        }
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

pub struct NesAudioCallback {
    output_buffer: SampleBuffer,
}

impl NesAudioCallback {
    pub fn get_output_buffer(&self) -> SampleBuffer {
        self.output_buffer.clone_ref()
    }
}

impl AudioCallback for NesAudioCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.output_buffer.output_samples(out);
    }
}

pub fn create_audio_device(sdl: &sdl2::Sdl) -> AudioDevice<NesAudioCallback> {
    let audio_subsystem = sdl.audio().unwrap();
    let audio_spec = AudioSpecDesired {
        freq: Some(48_000),
        channels: Some(1),
        samples: None,
    };
    audio_subsystem.open_playback(None, &audio_spec, |spec: AudioSpec| {
        NesAudioCallback {
            output_buffer: SampleBuffer::new(spec.freq as u32),
        }
    }).unwrap()
}
