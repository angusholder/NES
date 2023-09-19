use bitflags::bitflags;
use log::{info};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct InputState {
    p1_pressed: JoypadButtons,
    p2_pressed: JoypadButtons,

    is_polling: bool,
    joypad1_shift_register: u8,
    joypad2_shift_register: u8,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            p1_pressed: JoypadButtons::empty(),
            p2_pressed: JoypadButtons::empty(),
            is_polling: false,
            joypad1_shift_register: 0,
            joypad2_shift_register: 0,
        }
    }

    pub fn update_p1_key_state(&mut self, pressed: JoypadButtons) {
        Self::update_key_state(&mut self.p1_pressed, pressed, "P1");
    }

    pub fn update_p2_key_state(&mut self, pressed: JoypadButtons) {
        Self::update_key_state(&mut self.p2_pressed, pressed, "P2");
    }

    fn update_key_state(pressed: &mut JoypadButtons, new_pressed: JoypadButtons, name: &str) {
        let prev_pressed = *pressed;
        *pressed = new_pressed;
        let button_diff = prev_pressed.difference(new_pressed);
        if !button_diff.is_empty() {
            info!("{name} Pressed {button_diff:?}");
        }
    }

    pub fn write_joypad_strobe(&mut self, val: u8) {
        if val & 1 != 0 {
            self.is_polling = true;
        } else {
            if self.is_polling {
                self.is_polling = false;
                self.joypad1_shift_register = self.p1_pressed.bits;
                self.joypad2_shift_register = self.p2_pressed.bits;
            }
        }
    }

    pub fn read_joypad_1(&mut self) -> u8 {
        let next_bit = self.joypad1_shift_register & 1;
        self.joypad1_shift_register >>= 1;
        next_bit
    }

    pub fn read_joypad_2(&mut self) -> u8 {
        let next_bit = self.joypad2_shift_register & 1;
        self.joypad2_shift_register >>= 1;
        next_bit
    }
}

bitflags! {
    /// The current state of the joypad as a bitmask.
    /// https://www.nesdev.org/wiki/Standard_controller
    #[derive(Serialize, Deserialize)]
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
