use bitflags::bitflags;
use log::{info};

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

    pub fn write_joypad_strobe(&mut self, val: u8) {
        if val & 1 != 0 {
            self.is_polling = true;
        } else {
            if self.is_polling {
                self.is_polling = false;
                self.joypad1_shift_register = self.pressed.bits;
            }
        }
    }

    pub fn read_joypad_1(&mut self) -> u8 {
        let next_bit = self.joypad1_shift_register & 1;
        self.joypad1_shift_register >>= 1;
        next_bit
    }

    pub fn read_joypad_2(&mut self) -> u8 {
        // Controller 2 is not implemented, always return 0
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
