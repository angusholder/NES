use bitflags::bitflags;
use log::{info, warn};

pub struct InputState {
    pressed: JoypadButtons,

    is_polling: bool,
    joypad1_shift_register: u8,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: JoypadButtons::empty(),
            is_polling: false,
            joypad1_shift_register: 0,
        }
    }

    pub fn update_key_state(&mut self, pressed: JoypadButtons) {
        let prev_pressed = self.pressed;
        self.pressed = pressed;
        let button_diff = prev_pressed.difference(pressed);
        if !button_diff.is_empty() {
            info!("Pressed {button_diff:?}");
        }
    }

    pub fn handle_register_access(&mut self, addr: u16, val: u8, write: bool) -> u8 {
        if write && addr == JOYPAD_1 {
            if val & 1 != 0 {
                self.is_polling = true;
            } else {
                if self.is_polling {
                    self.is_polling = false;
                    self.joypad1_shift_register = self.pressed.bits;
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
}

bitflags! {
    /// The current state of the joypad as a bitmask.
    /// https://www.nesdev.org/wiki/Standard_controller
    pub struct JoypadButtons : u8 {
        const A = 1 << 0;
        const B = 1 << 1;
        const SELECT = 1 << 2;
        const START = 1 << 3;
        const UP = 1 << 4;
        const DOWN = 1 << 5;
        const LEFT = 1 << 6;
        const RIGHT = 1 << 7;
    }
}

pub const JOYPAD_1: u16 = 0x4016;
pub const JOYPAD_2: u16 = 0x4017;
