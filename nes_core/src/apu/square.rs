use crate::apu::CPU_FREQ;
use crate::apu::envelope::Envelope;
use crate::apu::length_counter::LengthCounter;
use crate::apu::sweep::Sweep;

pub struct SquareWave {
    duty_cycle: f32,
    period: u32,
    pub envelope: Envelope,
    pub sweep: Sweep,
    pub length_counter: LengthCounter,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SquareUnit {
    Pulse1,
    Pulse2,
}

impl SquareWave {
    pub fn new(unit: SquareUnit) -> SquareWave {
        let ones_complement = if unit == SquareUnit::Pulse1 { true } else { false };
        SquareWave {
            duty_cycle: 0.5,
            period: 0, // Range: 0-0x7FF / 0-2047 / 12.428KHz-54Hz
            envelope: Envelope::new(),
            sweep: Sweep::new(ones_complement),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn output_samples(
        &mut self,
        step_start_time_s: f64,
        step_duration_s: f64,
        output: &mut [u8],
    ) {
        let current_period = self.period;
        let target_period = self.sweep.calculate_target_period(current_period);
        let mute = self.sweep.should_mute(current_period);

        let period_s: f64 = (16 * (target_period + 1)) as f64 / CPU_FREQ as f64;
        let time_step = step_duration_s / output.len() as f64;
        for (i, sample) in output.iter_mut().enumerate() {
            let now_s = step_start_time_s + time_step * i as f64;
            let phase = (now_s / period_s) % 1.0;

            let mut volume = self.envelope.get_volume();
            if mute {
                volume = 0;
            }
            if self.length_counter.is_zero() {
                volume = 0;
            }
            if phase > self.duty_cycle as f64 {
                volume = 0;
            }

            *sample = volume;
        }
    }

    pub fn tick_length_and_swap(&mut self) {
        self.length_counter.tick();
        self.sweep.tick(&mut self.period);
    }

    // $4003/$4007
    // Side effect:
    //   The (square duty) sequencer is immediately restarted at the first value of the current
    //   sequence. The envelope is also restarted. The period divider is not reset.
    pub fn write_coarse_tune(&mut self, value: u8) {
        self.period = self.period & 0x00FF | ((value as u32 & 0x7) << 8);
        self.envelope.set_start_flag();
        self.length_counter.set_value(value >> 3);
    }

    // $4002/$4006
    pub fn write_fine_tune(&mut self, value: u8) {
        self.period = self.period & 0xFF00 | value as u32;
    }

    // $4000/$4004
    // DDLC VVVV
    pub fn write_control(&mut self, value: u8) {
        self.duty_cycle = match value >> 6 {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => unreachable!(),
        };

        self.length_counter.halt = value & 0b10_0000 != 0;
        self.envelope.loop_flag = 0b10_0000 & value != 0;
        self.envelope.constant_volume_flag = 0b01_0000 & value != 0;
        self.envelope.volume_or_envelope = value & 0b1111;
    }

    // $4001/$4005
    pub fn write_ramp(&mut self, value: u8) {
        self.sweep.enabled = value & 0x80 != 0;
        self.sweep.divider_period = ((value >> 4) & 0b111) + 1; // 1-8
        self.sweep.negate = value & 0x08 != 0;
        self.sweep.shift_count = value & 0b111;
        self.sweep.set_reload_flag();
    }

    fn get_target_period_and_volume(&self) -> (u32, u8) {
        let current_period = self.period;
        let target_period = self.sweep.calculate_target_period(current_period);
        let mut volume = self.envelope.get_volume();
        if self.sweep.should_mute(current_period) {
            volume = 0;
        }
        return (target_period, volume);
    }

    pub fn get_snapshot(&self) -> SquareWaveStateSnapshot {
        let (period, volume) = self.get_target_period_and_volume();
        SquareWaveStateSnapshot {
            duty_cycle: self.duty_cycle,
            period,
            volume,
        }
    }
}

/// Emit one of these whenever any parameters of the square wave change.
/// We don't currently track the phase of the wave, hopefully it's inconsequential?
pub struct SquareWaveStateSnapshot {
    duty_cycle: f32,
    period: u32,
    volume: u8, // 0-15
}
