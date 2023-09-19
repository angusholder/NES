use serde::{Deserialize, Serialize};
use crate::apu::length_counter::LengthCounter;
use crate::apu::linear_counter::LinearCounter;

#[derive(Serialize, Deserialize)]
pub struct TriangleWave {
    period: u32,
    timer: u32,
    sequence_pos: usize,
    pub length_counter: LengthCounter,
    pub linear_counter: LinearCounter,
}

impl TriangleWave {
    pub fn new() -> TriangleWave {
        TriangleWave {
            period: 0,
            timer: 0,
            sequence_pos: 0,
            length_counter: LengthCounter::new(),
            linear_counter: LinearCounter::new(),
        }
    }

    pub fn tick(&mut self) {
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.clock_waveform_generator();
            self.timer = self.period;
        }
    }

    fn clock_waveform_generator(&mut self) {
        self.sequence_pos = (self.sequence_pos + 1) % Self::OUTPUT_SEQUENCE.len();
    }

    const OUTPUT_SEQUENCE: [u8; 32] = [
        15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
        0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
    ];

    pub fn get_current_output(&self) -> u8 {
        let mut volume = Self::OUTPUT_SEQUENCE[self.sequence_pos];

        // At the expense of accuracy, [popping] can be eliminated in an emulator e.g. by halting the triangle channel when an ultrasonic frequency is set (a timer value less than 2).
        if self.period < 2 {
            volume = 0;
        }
        if self.length_counter.is_zero() {
            volume = 0;
        }
        if self.linear_counter.is_zero() {
            volume = 0;
        }

        volume
    }

    // $4008
    pub fn write_control(&mut self, value: u8) {
        self.linear_counter.control_flag = value & 0b1000_0000 != 0;
        self.length_counter.halt = value & 0b1000_0000 != 0;
        self.linear_counter.counter_reload_value = value & 0b0111_1111;
    }

    // $400A
    pub fn write_fine_tune(&mut self, value: u8) {
        self.period = self.period & 0xFF00 | (value as u32);
    }

    // $400B
    pub fn write_coarse_tune(&mut self, value: u8) {
        self.period = self.period & 0x00FF | ((value as u32 & 0x7) << 8);
        self.length_counter.set_value(value >> 3);
        // Side-effect: set the linear counter reload flag
        self.linear_counter.reload_flag = true;
    }
}
