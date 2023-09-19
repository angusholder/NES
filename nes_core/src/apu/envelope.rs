use serde::{Deserialize, Serialize};
use crate::apu::divider::Divider;

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    // Parameters:
    pub constant_volume_flag: bool,
    pub loop_flag: bool,
    pub volume_or_envelope: u8, // 0-15

    // State:
    start_flag: bool,
    divider: Divider,
    decay_level_counter: u8, // 0-15
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            constant_volume_flag: false,
            loop_flag: false,
            volume_or_envelope: 0,

            start_flag: true,
            divider: Divider::new(0),
            decay_level_counter: 0,
        }
    }

    /// https://www.nesdev.org/wiki/APU_Envelope
    pub fn tick(&mut self) {
        if !self.start_flag {
            if self.divider.tick(self.volume_or_envelope) {
                self.tick_decay_counter();
            }
        } else {
            self.start_flag = false;
            self.decay_level_counter = 15;
            self.divider.reset(self.volume_or_envelope);
        }
    }

    fn tick_decay_counter(&mut self) {
        if self.decay_level_counter > 0 {
            self.decay_level_counter -= 1;
        } else {
            if self.loop_flag {
                self.decay_level_counter = 15;
            }
        }
    }

    pub fn get_volume(&self) -> u8 {
        if self.constant_volume_flag {
            return self.volume_or_envelope;
        } else {
            return self.decay_level_counter;
        }
    }

    /// Triggered by writing $4003/$4007/$400F
    pub fn set_start_flag(&mut self) {
        self.start_flag = true;
    }
}
