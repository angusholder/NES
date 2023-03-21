use crate::apu::CPU_FREQ;

pub struct SquareWave {
    volume: u8,

    duty_cycle: f32,
    period: u32,
}

impl SquareWave {
    pub fn new() -> SquareWave {
        SquareWave {
            volume: 15,
            duty_cycle: 0.5,
            period: 0, // Range: 0-0x7FF / 0-2047 / 12.428KHz-54Hz
        }
    }

    pub fn output_samples(
        &mut self,
        step_start_time_s: f64,
        step_duration_s: f64,
        output: &mut [u8],
    ) {
        if self.period < 8 {
            output.fill(0);
            // All zeroes
            return;
        }

        let period_s: f64 = (16 * (self.period + 1)) as f64 / CPU_FREQ as f64;
        let time_step = step_duration_s / output.len() as f64;
        for (i, sample) in output.iter_mut().enumerate() {
            let now_s = step_start_time_s + time_step * i as f64;
            let phase = (now_s / period_s) % 1.0;
            if phase <= self.duty_cycle as f64 { // duty_cycle
                *sample = self.volume;
            } else {
                *sample = 0;
            };
        }
    }

    // $4003/$4007
    pub fn write_coarse_tune(&mut self, value: u8) {
        // TODO: Reset the phase
        self.period = self.period & 0x00FF | ((value as u32 & 0x7) << 8);
        // TODO: Reset length counter
    }

    // $4002/$4006
    pub fn write_fine_tune(&mut self, value: u8) {
        self.period = self.period & 0xFF00 | value as u32;
    }

    // $4000/$4004
    pub fn write_control(&mut self, value: u8) {
        self.duty_cycle = match value >> 6 {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => unreachable!(),
        };
        self.volume = value & 0xF;
    }

    // $4001/$4005
    pub fn write_ramp(&mut self, _value: u8) {

    }
}
