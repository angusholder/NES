use crate::apu::envelope::Envelope;
use crate::apu::length_counter::LengthCounter;
use crate::apu::sweep::Sweep;

pub struct SquareWave {
    timer: u32,
    duty_cycle_pos: u8,
    duty_cycle_mask: u8,
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
            timer: 0,
            duty_cycle_pos: 1,
            duty_cycle_mask: 0b00000001,
            period: 0, // Range: 0-0x7FF / 0-2047 / 12.428KHz-54Hz
            envelope: Envelope::new(),
            sweep: Sweep::new(ones_complement),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn get_current_output(&self) -> u8 {
        let mut volume: u8 = self.envelope.get_volume();

        if self.sweep.should_mute(self.period) {
            volume = 0;
        }
        if self.length_counter.is_zero() {
            volume = 0;
        }
        if self.duty_cycle_pos & self.duty_cycle_mask == 0 {
            volume = 0;
        }

        volume
    }

    pub fn tick(&mut self) {
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.clock_sequencer();
            self.timer = self.sweep.calculate_target_period(self.period);
        }
    }

    fn clock_sequencer(&mut self) {
        self.duty_cycle_pos = self.duty_cycle_pos.rotate_right(1);
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
        self.duty_cycle_mask = match value >> 6 {
            0 => 0b00000001, // 0 1 0 0 0 0 0 0 (12.5%)
            1 => 0b00000011, // 0 1 1 0 0 0 0 0 (25%)
            2 => 0b00001111, // 0 1 1 1 1 0 0 0 (50%)
            3 => 0b11111100, // 1 0 0 1 1 1 1 1 (25% negated)
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
}
