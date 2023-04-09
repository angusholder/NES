use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::panic::catch_unwind;
use std::path::Path;
use std::time::{Duration, Instant};
use log::info;
use minifb::{Key, KeyRepeat, Menu, MENU_KEY_CTRL, Scale, Window, WindowOptions};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired};
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::messagebox::{ButtonData, MessageBoxButtonFlag, MessageBoxFlag, show_message_box};
use nes_core::apu::{AudioChannels, SampleBuffer};
use nes_core::cartridge;
use nes_core::input::JoypadButtons;
use nes_core::nes::NES;
use nes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_PIXELS};

const TRACE_FILE: bool = false;

fn main() {
    let mut log_builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    if TRACE_FILE {
        log_builder.target(StdoutAndFileTarget::new(File::create("trace.txt").unwrap()));
    }
    log_builder.init();

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
    let controller_subsystem = sdl_context.game_controller().unwrap();

    let mut window = Window::new("NES Emulator", SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize, WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    })?;
    window.limit_update_rate(Some(Duration::from_micros(16_666)));

    let mut file_menu = Menu::new("File")?;
    file_menu.add_item("Open", ACTION_OPEN).shortcut(Key::O, MENU_KEY_CTRL).build();
    window.add_menu(&file_menu);

    let mut audio_device: AudioDevice<NesAudioCallback> = create_audio_device(&sdl_context);
    info!("Got audio device: {:?}", audio_device.spec());

    let keymap: Keymap = get_key_map();

    let mut controller_mappings = &include_bytes!("../gamecontrollerdb.txt")[..];
    controller_subsystem.load_mappings_from_read(&mut controller_mappings).unwrap();
    let mut game_controller: Option<GameController> = None;

    let mut event_pump: EventPump = sdl_context.event_pump()?;

    let mut frame_stats = FrameStats::new();
    let mut nes: Option<Box<NES>> = None;
    let mut paused = false;
    while window.is_open() {
        let start_time = Instant::now();

        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            paused = !paused;
        }
        let mut toggle_channel = |channel: AudioChannels| {
            if let Some(nes) = nes.as_mut() {
                nes.apu.toggle_channel(channel);
            }
        };
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) { toggle_channel(AudioChannels::SQUARE1); }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) { toggle_channel(AudioChannels::SQUARE2); }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) { toggle_channel(AudioChannels::TRIANGLE); }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) { toggle_channel(AudioChannels::NOISE); }
        if window.is_key_pressed(Key::Key5, KeyRepeat::No) { toggle_channel(AudioChannels::DMC); }

        match window.is_menu_pressed().unwrap_or(usize::MAX) {
            ACTION_OPEN => {
                handle_open_file(&mut audio_device, &mut nes);
            }
            _ => {}
        }
        for event in event_pump.poll_iter() {
            match event {
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
                _ => {}
            }
        }

        let has_focus = window.is_active();
        if !paused && has_focus {
            if let Some(nes) = &mut nes {
                nes.input.update_key_state(get_pressed_buttons(&window, &keymap, game_controller.as_ref()));

                nes.simulate_frame();

                let mut display_buffer = [0u32; SCREEN_PIXELS];
                nes.ppu.output_display_buffer_u32_argb(&mut display_buffer);
                window.update_with_buffer(&display_buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)?;
            } else {
                window.update_with_buffer(&[0; SCREEN_PIXELS], SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)?;
            }
        } else {
            window.update();
        }

        let pause_text = if paused { " - PAUSED" } else { "" };
        window.set_title(&format!("NES Emulator - {:.2}ms{}", frame_stats.get_avg_frame_time_ms(), pause_text));
        let frame_time = start_time.elapsed();
        frame_stats.add_reading(frame_time);
    }

    Ok(())
}

fn handle_open_file(audio_device: &mut AudioDevice<NesAudioCallback>, nes: &mut Option<Box<NES>>) {
    let Some(filename) = rfd::FileDialog::new()
        .set_title("Open NES ROM")
        .add_filter(".NES", &["nes"])
        .pick_file() else { return; };

    match load_nes_system(&filename) {
        Ok(mut new_nes) => {
            let mut sample_buffer = audio_device.lock().get_output_buffer();
            sample_buffer.clear();
            new_nes.apu.attach_output_device(sample_buffer);
            audio_device.resume();
            *nes = Some(new_nes);
        }
        Err(e) => {
            display_error_dialog("Failed to load the ROM", &e.to_string());
        }
    }
}

const ACTION_OPEN: usize = 1;

fn load_nes_system(
    filename: &Path,
) -> Result<Box<NES>, Box<dyn Error>> {
    let cart = cartridge::parse_rom(filename)?;
    let mut nes = Box::new(NES::from_cart(cart));
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

type Keymap = HashMap<Key, JoypadButtons>;

fn get_key_map() -> Keymap {
    let mut map = HashMap::new();
    map.insert(Key::Z, JoypadButtons::A);
    map.insert(Key::X, JoypadButtons::B);
    map.insert(Key::A, JoypadButtons::SELECT);
    map.insert(Key::S, JoypadButtons::START);
    map.insert(Key::Enter, JoypadButtons::START);
    map.insert(Key::Up, JoypadButtons::UP);
    map.insert(Key::Down, JoypadButtons::DOWN);
    map.insert(Key::Left, JoypadButtons::LEFT);
    map.insert(Key::Right, JoypadButtons::RIGHT);
    map
}

pub fn get_pressed_buttons(window: &Window, keymap: &Keymap, controller: Option<&GameController>) -> JoypadButtons {
    let mut pressed = JoypadButtons::empty();
    for (key, button) in keymap.iter() {
        if window.is_key_down(*key) {
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
        samples: Some(735 * 3),
    };
    audio_subsystem.open_playback(None, &audio_spec, |spec: AudioSpec| {
        NesAudioCallback {
            output_buffer: SampleBuffer::new(spec.freq as u32),
        }
    }).unwrap()
}

struct StdoutAndFileTarget {
    file: BufWriter<File>,
}

impl StdoutAndFileTarget {
    fn new(file: File) -> env_logger::Target {
        env_logger::Target::Pipe(Box::new(StdoutAndFileTarget {
            file: BufWriter::new(file),
        }))
    }
}

impl io::Write for StdoutAndFileTarget {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write_all(buf)?;
        io::stdout().write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
