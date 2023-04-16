use crate::apu::CPU_FREQ;
use crate::apu::length_counter::LengthCounter;
use crate::apu::linear_counter::LinearCounter;

pub struct TriangleWave {
    period: u32,
    pub length_counter: LengthCounter,
    pub linear_counter: LinearCounter,
}

impl TriangleWave {
    pub fn new() -> TriangleWave {
        TriangleWave {
            period: 0,
            length_counter: LengthCounter::new(),
            linear_counter: LinearCounter::new(),
        }
    }

    pub fn output_samples(
        &mut self,
        step_start_time_s: f64,
        step_duration_s: f64,
        output: &mut [u8],
    ) {
        let period_s: f64 = (32 * (self.period + 1)) as f64 / CPU_FREQ as f64;
        let time_step = step_duration_s / output.len() as f64;
        for (i, sample) in output.iter_mut().enumerate() {
            let now_s = step_start_time_s + time_step * i as f64;
            let scaled: f64  = now_s / period_s * 4.0;
            // Number between 0 and 3 - which of the 4 sections of the triangle wave are we in
            let cycle_phase = scaled as i64 % 4;
            // Number between 0 and 1 - how far through a single section are we
            let cycle_offset = (scaled % 1.0) as f32;

            let step_m1_1 = match cycle_phase {
                0 => cycle_offset, // 0 to 1
                1 => 1.0 - cycle_offset, // 1 to 0
                2 => -cycle_offset, // 0 to -1
                3 => -1.0 + cycle_offset, // -1 to 0
                _ => unreachable!(),
            };
            let step_0_1 = (step_m1_1 + 1.0) / 2.0;
            let mut volume = if step_0_1 >= 1.0 {
                15
            } else if step_0_1 <= 0.0 {
                0
            } else {
                (step_0_1 * 16.0).floor() as u8
            };

            if self.period < 2 {
                volume = 0;
            }
            if self.length_counter.is_zero() {
                volume = 0;
            }
            if self.linear_counter.is_zero() {
                volume = 0;
            }

            *sample = volume;
        }
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
