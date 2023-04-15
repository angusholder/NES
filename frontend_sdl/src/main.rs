use std::collections::{VecDeque};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use log::info;
use minifb::{Key, KeyRepeat, Menu, MENU_KEY_CTRL, Scale, ScaleMode, Window, WindowOptions};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired};
use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::messagebox::{ButtonData, MessageBoxButtonFlag, MessageBoxFlag, show_message_box};
use nes_core::apu::{AudioChannels, SampleBuffer};
use nes_core::cartridge;
use nes_core::input::JoypadButtons;
use nes_core::nes::{Interrupt, NES};
use nes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_PIXELS};

const TRACE_FILE: bool = false;

fn main() {
    let mut log_builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    if TRACE_FILE {
        log_builder.target(StdoutAndFileTarget::new(File::create("trace.txt").unwrap()));
    }
    log_builder.init();

    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // The NES code panicked - probably an instruction/system not implemented yet, or a bug
        let mut err_msg: String = "Unknown error".to_string();
        if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
            err_msg = msg.clone();
        } else if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
            err_msg = msg.to_string();
        }
        display_error_dialog("Unexpected runtime error", &err_msg);
        // Once the user dismisses our dialog, delegate to the default panic handler so it prints to
        // stderr then kills the process.
        default_panic_hook(panic_info);
        std::process::exit(101);
    }));

    let result = main_loop();
    match result {
        Ok(()) => {}
        Err(e) => {
            // An explicit error returned
            display_error_dialog("Unexpected error", &e.to_string());
        }
    }
}

fn main_loop() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;
    let controller_subsystem = sdl_context.game_controller().unwrap();

    let mut window = Window::new("NES Emulator", SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize, WindowOptions {
        scale: Scale::X2,
        resize: true,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..WindowOptions::default()
    })?;
    window.set_background_color(0x20, 0x20, 0x20);
    window.limit_update_rate(Some(Duration::from_micros(16_666)));

    let mut file_menu = Menu::new("File")?;
    file_menu.add_item("Open", ACTION_OPEN).shortcut(Key::O, MENU_KEY_CTRL).build();
    file_menu.add_item("Reset", ACTION_RESET).build();
    file_menu.add_item("Stop", ACTION_STOP).build();
    window.add_menu(&file_menu);

    let audio_device: AudioDevice<NesAudioCallback> = create_audio_device(&sdl_context);
    info!("Got audio device: {:?}", audio_device.spec());

    let mut controller_mappings = &include_bytes!("../gamecontrollerdb.txt")[..];
    controller_subsystem.load_mappings_from_read(&mut controller_mappings).unwrap();
    let mut game_controller: Option<GameController> = None;

    let mut event_pump: EventPump = sdl_context.event_pump()?;

    let mut frame_stats = FrameStats::new();
    let mut app = App::new(audio_device);
    while window.is_open() {
        let start_time = Instant::now();

        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            app.toggle_pause();
        }
        let mut toggle_channel = |channel: AudioChannels| {
            if let Some(nes) = app.nes.as_mut() {
                nes.apu.toggle_channel(channel);
            }
        };
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) { toggle_channel(AudioChannels::SQUARE1); }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) { toggle_channel(AudioChannels::SQUARE2); }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) { toggle_channel(AudioChannels::TRIANGLE); }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) { toggle_channel(AudioChannels::NOISE); }
        if window.is_key_pressed(Key::Key5, KeyRepeat::No) { toggle_channel(AudioChannels::DMC); }

        match window.is_menu_pressed().unwrap_or(usize::MAX) {
            ACTION_OPEN => app.open_file_dialog(),
            ACTION_STOP => app.close_rom(),
            ACTION_RESET => app.reset(),
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
        if !app.paused && has_focus {
            if let Some(nes) = app.nes.as_mut() {
                nes.input.update_p1_key_state(get_pressed_buttons(&window, game_controller.as_ref()));
                nes.input.update_p2_key_state(JoypadButtons::empty()); // Not implemented

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

        let pause_text = if app.paused { " - PAUSED" } else { "" };
        let mut game_text = String::new();
        if let Some(filename) = app.rom_filename.as_ref().map(|i| i.file_name()).flatten() {
            game_text = format!("- {}", filename.to_string_lossy());
        }
        window.set_title(&format!("NES Emulator{game_text} - {:.2}ms{}", frame_stats.get_avg_frame_time_ms(), pause_text));
        let frame_time = start_time.elapsed();
        frame_stats.add_reading(frame_time);
    }

    Ok(())
}

struct App {
    audio_device: AudioDevice<NesAudioCallback>,
    nes: Option<Box<NES>>,
    rom_filename: Option<PathBuf>,
    paused: bool,
}

impl App {
    fn new(audio_device: AudioDevice<NesAudioCallback>) -> App {
        App {
            audio_device,
            nes: None,
            rom_filename: None,
            paused: false,
        }
    }

    fn open_file_dialog(&mut self) {
        let Some(filename) = rfd::FileDialog::new()
            .set_title("Open NES ROM")
            .add_filter(".NES", &["nes"])
            .pick_file() else { return; };

        self.load_rom(filename);
    }

    fn load_rom(&mut self, rom_filename: PathBuf) {
        match load_nes_system(&rom_filename) {
            Ok(mut nes) => {
                let mut sample_buffer = self.audio_device.lock().get_output_buffer();
                sample_buffer.clear();
                nes.apu.attach_output_device(sample_buffer);
                self.audio_device.resume();
                self.nes = Some(nes);
                self.rom_filename = Some(rom_filename);
            }
            Err(e) => {
                display_error_dialog("Failed to load the ROM", &e.to_string());
            }
        }
    }

    fn close_rom(&mut self) {
        self.nes = None;
        self.rom_filename = None;
        self.audio_device.pause();
        self.audio_device.lock().get_output_buffer().clear();
    }

    fn reset(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            nes.interrupt(Interrupt::RESET);
        }
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

const ACTION_OPEN: usize = 1;
const ACTION_STOP: usize = 2;
const ACTION_RESET: usize = 3;

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

pub fn get_pressed_buttons(window: &Window, controller: Option<&GameController>) -> JoypadButtons {
    let mut pressed = JoypadButtons::empty();

    let keymap = &[
        (Key::Z, JoypadButtons::A),
        (Key::X, JoypadButtons::B),
        (Key::A, JoypadButtons::SELECT),
        (Key::S, JoypadButtons::START),
        (Key::Enter, JoypadButtons::START),
        (Key::Up, JoypadButtons::UP),
        (Key::Down, JoypadButtons::DOWN),
        (Key::Left, JoypadButtons::LEFT),
        (Key::Right, JoypadButtons::RIGHT),
    ];
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
