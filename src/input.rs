use std::collections::{HashMap, HashSet};
use std::ops::BitOr;
use log::{info, warn};
use sdl2::EventPump;
use sdl2::keyboard::Scancode;

pub struct InputState {
    key_map: HashMap<Scancode, JoypadButton>,
    pressed: HashSet<JoypadButton>,

    is_polling: bool,
    joypad1_shift_register: u8,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key_map: get_key_map(),
            pressed: HashSet::with_capacity(8),

            is_polling: false,
            joypad1_shift_register: 0,
        }
    }

    pub fn update_key_state(&mut self, event_pump: &EventPump) {
        let prev_pressed = self.pressed.clone();
        self.pressed.clear();
        for (scan_code, button) in self.key_map.iter() {
            if event_pump.keyboard_state().is_scancode_pressed(*scan_code) {
                self.pressed.insert(*button);
            }
        }
        for new_button in self.pressed.difference(&prev_pressed) {
            info!("Pressed {new_button:?}");
        }
    }

    pub fn handle_register_access(&mut self, addr: u16, val: u8, write: bool) -> u8 {
        if write && addr == JOYPAD_1 {
            if val & 1 != 0 {
                self.is_polling = true;
            } else {
                if self.is_polling {
                    self.is_polling = false;
                    self.joypad1_shift_register = self.get_button_bitmask();
                }
            }
            return 0;
        }
        if write && addr == JOYPAD_2 {
            // Ignore
            return 0;
        }
        if !write && addr == JOYPAD_1 {
            let next_bit = self.joypad1_shift_register & 1;
            self.joypad1_shift_register >>= 1;
            return next_bit;
        }
        if !write && addr == JOYPAD_2 {
            // Controller 2 is not implemented, always return 0
            return 0;
        }

        warn!("Unhandled controller access: {addr:04X}/{write}/{val}");
        0
    }

    /// Returns the current state of the joypad as a bitmask.
    /// https://www.nesdev.org/wiki/Standard_controller
    fn get_button_bitmask(&self) -> u8 {
        self.pressed
            .iter()
            .map(|b| b.get_bit())
            .fold(0, u8::bitor)
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
enum JoypadButton {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
}

impl JoypadButton {
    fn get_bit(&self) -> u8 {
        1 << *self as u8
    }
}

fn get_key_map() -> HashMap<Scancode, JoypadButton> {
    let mut map = HashMap::new();
    map.insert(Scancode::Z, JoypadButton::A);
    map.insert(Scancode::X, JoypadButton::B);
    map.insert(Scancode::A, JoypadButton::Select);
    map.insert(Scancode::S, JoypadButton::Start);
    map.insert(Scancode::Return, JoypadButton::Start);
    map.insert(Scancode::Up, JoypadButton::Up);
    map.insert(Scancode::Down, JoypadButton::Down);
    map.insert(Scancode::Left, JoypadButton::Left);
    map.insert(Scancode::Right, JoypadButton::Right);
    map
}

pub const JOYPAD_1: u16 = 0x4016;
pub const JOYPAD_2: u16 = 0x4017;
