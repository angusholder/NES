use std::collections::{HashMap};
use bitflags::bitflags;
use log::{info, warn};
use sdl2::EventPump;
use sdl2::keyboard::Scancode;

pub struct InputState {
    key_map: HashMap<Scancode, JoypadButtons>,
    pressed: JoypadButtons,

    is_polling: bool,
    joypad1_shift_register: u8,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key_map: get_key_map(),
            pressed: JoypadButtons::empty(),

            is_polling: false,
            joypad1_shift_register: 0,
        }
    }

    pub fn update_key_state(&mut self, event_pump: &EventPump) {
        let prev_pressed = self.pressed;
        self.pressed = JoypadButtons::empty();
        for (scan_code, button) in self.key_map.iter() {
            if event_pump.keyboard_state().is_scancode_pressed(*scan_code) {
                self.pressed.insert(*button);
            }
        }
        info!("Pressed {:?}", self.pressed.difference(prev_pressed));
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
        self.pressed.bits()
    }
}

bitflags! {
    pub struct JoypadButtons : u8 {
        const A = 0;
        const B = 1;
        const SELECT = 2;
        const START = 3;
        const UP = 4;
        const DOWN = 5;
        const LEFT = 6;
        const RIGHT = 7;
    }
}

fn get_key_map() -> HashMap<Scancode, JoypadButtons> {
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

pub const JOYPAD_1: u16 = 0x4016;
pub const JOYPAD_2: u16 = 0x4017;
