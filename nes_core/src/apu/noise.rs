use serde::{Deserialize, Serialize};
use crate::apu::envelope::Envelope;
use crate::apu::length_counter::LengthCounter;

/// https://www.nesdev.org/wiki/APU_Noise
#[derive(Serialize, Deserialize)]
pub struct Noise {
    period: u32,
    timer: u32,
    feedback_bit_6: bool,
    shift_register: u16, // 15 bits

    pub envelope: Envelope,
    pub length_counter: LengthCounter,
}

impl Noise {
    const PERIOD_LUT: [u32; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

    pub fn new() -> Noise {
        Noise {
            period: Self::PERIOD_LUT[0],
            timer: 0,
            feedback_bit_6: false,
            shift_register: 1,

            envelope: Envelope::new(),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn write_control(&mut self, value: u8) {
        self.envelope.volume_or_envelope = value & 0xF;
        self.envelope.constant_volume_flag = value & 0x10 != 0;
        self.length_counter.halt = value & 0x20 != 0;
    }

    pub fn write_noise_freq1(&mut self, value: u8) {
        let period_index = (value & 0xF) as usize;
        self.period = Self::PERIOD_LUT[period_index];
        self.feedback_bit_6 = value & 0x80 != 0; // otherwise bit 1
    }

    pub fn write_noise_freq2(&mut self, value: u8) {
        self.length_counter.set_value(value >> 3);
        self.envelope.set_start_flag();
    }

    pub fn get_current_output(&self) -> u8 {
        let mut volume = self.envelope.get_volume();

        if self.shift_register & 1 == 1 {
            volume = 0;
        }
        if self.length_counter.is_zero() {
            volume = 0;
        }

        volume
    }

    pub fn tick(&mut self) {
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.clock_shift_register();
            self.timer = self.period;
        }
    }

    fn clock_shift_register(&mut self) {
        let mut sr = self.shift_register;

        let shift_amt = if self.feedback_bit_6 { 6 } else { 1 };
        let feedback = (sr & 1) ^ (sr >> shift_amt & 1);

        sr >>= 1;

        sr |= feedback << 14;

        self.shift_register = sr;
    }
}
