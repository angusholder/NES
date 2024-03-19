use std::rc::Rc;
use bitflags::bitflags;
use log::{info};
use crate::apu::dmc::DMC;

mod square;
mod triangle;
mod noise;
mod envelope;
mod sweep;
mod divider;
mod length_counter;
mod linear_counter;
mod dmc;

use crate::apu::noise::Noise;
use crate::apu::square::{SquareUnit, SquareWave};
use crate::apu::triangle::TriangleWave;
use crate::mapper;
use crate::mapper::Mapper;
use crate::nes::{CYCLES_PER_FRAME, InterruptSource, Signals};

pub struct APU {
    square_wave1: SquareWave,
    square_wave2: SquareWave,
    triangle_wave: TriangleWave,
    noise: Noise,
    dmc: DMC,

    /// The user can override to mute a channel that the game has enabled.
    host_enabled_channels: AudioChannels,

    irq_inhibit: bool,
    frame_counter_mode: FrameCountMode,

    mixed_samples: Vec<f32>,
    cycles_between_samples: f64,
    next_cycle_to_sample: u64,
    sample_count: u64,

    apu_cycle: u64,
    signals: Rc<Signals>,

    low_pass_filter: IIRFilter,
}

#[derive(PartialEq, Debug)]
enum FrameCountMode {
    Step4,
    Step5,
}

bitflags! {
    pub struct AudioChannels : u8 {
        const SQUARE1 = 0x01;
        const SQUARE2 = 0x02;
        const TRIANGLE = 0x04;
        const NOISE = 0x08;
        const DMC = 0x10;
    }
}

const SAMPLES_PER_FRAME: u32 = 735;
const SAMPLES_PER_SECOND: u32 = SAMPLES_PER_FRAME * 60;
impl APU {
    pub fn new(mapper: Rc<Mapper>, signals: Rc<Signals>) -> APU {
        APU {
            square_wave1: SquareWave::new(SquareUnit::Pulse1),
            square_wave2: SquareWave::new(SquareUnit::Pulse2),
            triangle_wave: TriangleWave::new(),
            noise: Noise::new(),
            dmc: DMC::new(mapper, Rc::clone(&signals)),

            host_enabled_channels: AudioChannels::all(),

            irq_inhibit: false,
            frame_counter_mode: FrameCountMode::Step4,

            mixed_samples: Vec::with_capacity(SAMPLES_PER_FRAME as usize),

            cycles_between_samples: (CYCLES_PER_FRAME as f64 / SAMPLES_PER_FRAME as f64),
            next_cycle_to_sample: 0,
            sample_count: 0,

            apu_cycle: 0,
            signals,

            // The NES hardware follows the DACs with a surprisingly involved circuit that adds several low-pass and high-pass filters:
            // - A first-order high-pass filter at 90 Hz
            // - Another first-order high-pass filter at 440 Hz
            // - A first-order low-pass filter at 14 kHz
            low_pass_filter: IIRFilter::new(14_000.0, 1.0 / SAMPLES_PER_SECOND as f32),
        }
    }

    pub fn step_cycle(&mut self, cpu_cycles: u64) {
        self.triangle_wave.tick();

        // All APU components other than triangle run at half the CPU clock rate, so skip them every other call.
        if cpu_cycles & 1 == 0 {
            self.square_wave1.tick();
            self.square_wave2.tick();
            self.noise.tick();
            self.dmc.tick();

            self.apu_cycle += 1;

            // See https://www.nesdev.org/wiki/APU_Frame_Counter
            match self.apu_cycle {
                3728 => {
                    self.tick_envelope_and_triangle();
                }
                7456 => {
                    self.tick_envelope_and_triangle();
                    self.tick_length_counters_and_sweep();
                }
                11185 => {
                    self.tick_envelope_and_triangle();
                }
                14914 if self.frame_counter_mode == FrameCountMode::Step4 => {
                    self.tick_envelope_and_triangle();
                    self.tick_length_counters_and_sweep();
                    self.trigger_irq();
                    self.apu_cycle = 0;
                }
                18640 if self.frame_counter_mode == FrameCountMode::Step5 => {
                    self.tick_envelope_and_triangle();
                    self.tick_length_counters_and_sweep();
                    self.apu_cycle = 0;
                }
                _ => {}
            }
        }

        if cpu_cycles >= self.next_cycle_to_sample {
            self.record_sample();
        }
    }

    fn record_sample(&mut self) {
        let mut sample: f32 = self.get_current_output();
        sample = self.low_pass_filter.filter_sample(sample);
        self.mixed_samples.push(sample);
        self.sample_count += 1;

        let next_cycle_to_sample: f64 = self.sample_count as f64 * self.cycles_between_samples;
        self.next_cycle_to_sample = next_cycle_to_sample.round() as u64;
    }

    fn tick_envelope_and_triangle(&mut self) {
        self.square_wave1.envelope.tick();
        self.square_wave2.envelope.tick();
        self.triangle_wave.linear_counter.tick();
        self.noise.envelope.tick();
    }

    fn tick_length_counters_and_sweep(&mut self) {
        self.square_wave1.tick_length_and_swap();
        self.square_wave2.tick_length_and_swap();
        self.triangle_wave.length_counter.tick();
        self.noise.length_counter.tick();
    }

    fn trigger_irq(&mut self) {
        if self.irq_inhibit {
            return;
        }
        self.signals.request_interrupt(InterruptSource::APU_FRAME_COUNTER);
    }

    fn get_current_output(&self) -> f32 {
        Self::mix_channels(
            self.square_wave1.get_current_output(),
            self.square_wave2.get_current_output(),
            self.triangle_wave.get_current_output(),
            self.noise.get_current_output(),
            self.dmc.get_current_output(),
            self.host_enabled_channels,
        )
    }

    fn mix_channels(
        mut pulse1: u8, // 0 to 15 (4-bit)
        mut pulse2: u8, // 0 to 15 (4-bit)
        mut triangle: u8, // 0 to 15 (4-bit)
        mut noise: u8, // 0 to 15 (4-bit)
        mut dmc: u8, // 0 to 127 (7-bit)
        enabled: AudioChannels,
    ) -> f32 {
        if !enabled.contains(AudioChannels::SQUARE1) { pulse1 = 0; }
        if !enabled.contains(AudioChannels::SQUARE2) { pulse2 = 0; }
        if !enabled.contains(AudioChannels::TRIANGLE) { triangle = 0; }
        if !enabled.contains(AudioChannels::NOISE) { noise = 0; }
        if !enabled.contains(AudioChannels::DMC) { dmc = 0; }

        // Lookup table from https://www.nesdev.org/wiki/APU_Mixer
        static PULSE_OUT: [f32; 31] = {
            let mut result = [0.0f32; 31];
            let mut n = 0;
            while n < result.len() {
                result[n] = 95.52 / (8128.0 / (n as f32) + 100.0);
                n += 1;
            }
            result
        };

        // Lookup table from https://www.nesdev.org/wiki/APU_Mixer
        static TND_OUT: [f32; 203] = {
            let mut result = [0.0f32; 203];
            let mut n = 0;
            while n < result.len() {
                result[n] = 163.67 / (24329.0 / (n as f32) + 100.0);
                n += 1;
            }
            result
        };

        // Mixing formula from here: https://www.nesdev.org/wiki/APU_Mixer
        let pulse_out = PULSE_OUT[(pulse1 + pulse2) as usize];
        let tnd_out = TND_OUT[(3 * triangle + 2 * noise + dmc) as usize];
        let output = pulse_out + tnd_out;
        // Output is between 0 and 1
        output
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                self.read_status_register()
            }
            _ => {
                mapper::out_of_bounds_read("APU", addr)
            }
        }
    }

    fn read_status_register(&mut self) -> u8 {
        // https://www.nesdev.org/wiki/APU#Status_($4015)

        let mut status = 0u8;

        if self.signals.is_active(InterruptSource::APU_FRAME_COUNTER) {
            status |= 0x40;
            self.signals.acknowledge_interrupt(InterruptSource::APU_FRAME_COUNTER);
        }
        if self.signals.is_active(InterruptSource::APU_DMC) {
            status |= 0x80;
        }
        if !self.square_wave1.length_counter.is_zero() {
            status |= 0b0001;
        }
        if !self.square_wave2.length_counter.is_zero() {
            status |= 0b0010;
        }
        if !self.triangle_wave.length_counter.is_zero() {
            status |= 0b0100;
        }
        if !self.noise.length_counter.is_zero() {
            status |= 0b1000;
        }
        if self.dmc.has_bytes_remaining() {
            status |= 0b1_0000;
        }

        status
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => self.square_wave1.write_control(value),
            0x4001 => self.square_wave1.write_ramp(value),
            0x4002 => self.square_wave1.write_fine_tune(value),
            0x4003 => self.square_wave1.write_coarse_tune(value),

            0x4004 => self.square_wave2.write_control(value),
            0x4005 => self.square_wave2.write_ramp(value),
            0x4006 => self.square_wave2.write_fine_tune(value),
            0x4007 => self.square_wave2.write_coarse_tune(value),

            0x4008 => self.triangle_wave.write_control(value),
            0x400A => self.triangle_wave.write_fine_tune(value),
            0x400B => self.triangle_wave.write_coarse_tune(value),

            0x400C => self.noise.write_control(value),
            0x400E => self.noise.write_noise_freq1(value),
            0x400F => self.noise.write_noise_freq2(value),

            0x4010 => self.dmc.write_control(value),
            0x4011 => self.dmc.write_direct_load(value),
            0x4012 => self.dmc.write_sample_address(value),
            0x4013 => self.dmc.write_sample_length(value),

            0x4015 => self.write_status_register(value),
            0x4017 => self.write_frame_counter(value),

            _ => {}
        }
    }

    pub fn write_status_register(&mut self, value: u8) {
        let channels = AudioChannels::from_bits_truncate(value);
        self.square_wave1.length_counter.set_channel_enabled(channels.contains(AudioChannels::SQUARE1));
        self.square_wave2.length_counter.set_channel_enabled(channels.contains(AudioChannels::SQUARE2));
        self.triangle_wave.length_counter.set_channel_enabled(channels.contains(AudioChannels::TRIANGLE));
        self.noise.length_counter.set_channel_enabled(channels.contains(AudioChannels::NOISE));
        self.dmc.set_channel_enabled(channels.contains(AudioChannels::DMC));
        // Writing to this register clears the DMC interrupt flag.
        self.signals.acknowledge_interrupt(InterruptSource::APU_DMC);
    }

    fn write_frame_counter(&mut self, value: u8) {
        self.irq_inhibit = value & 0x40 != 0;
        self.frame_counter_mode = if value & 0x80 == 0 { FrameCountMode::Step4 } else { FrameCountMode::Step5 };
        self.apu_cycle = 0;
        if self.frame_counter_mode == FrameCountMode::Step5 {
            // If the mode flag is set, then both "quarter frame" and "half frame" signals are also generated
            self.tick_envelope_and_triangle();
            self.tick_length_counters_and_sweep();
        }
    }

    pub fn toggle_channel(&mut self, channel: AudioChannels) {
        self.host_enabled_channels.toggle(channel);
        let state = if self.host_enabled_channels.contains(channel) { "on" } else { "off" };
        info!("Toggled channel {channel:?} to {state}")
    }

    pub fn output_samples(&mut self, output: impl FnOnce(&[f32])) {
        // TODO: Find out why some frames have one fewer or one more sample than they should.
        // if self.mixed_samples.len() != SAMPLES_PER_FRAME as usize {
        //     warn!("Expected {SAMPLES_PER_FRAME} samples, got {}", self.mixed_samples.len());
        // }
        output(&self.mixed_samples[..]);
        self.mixed_samples.clear();
    }
}

// https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
pub struct IIRFilter {
    last_sample_output: f32,
    cutoff_freq: f32,
    sample_period: f32,
    alpha: f32,
}

impl IIRFilter {
    pub fn new(cutoff_freq: f32, sample_period: f32) -> IIRFilter {
        use std::f32::consts::PI;
        let alpha = (2.0 * PI * sample_period * cutoff_freq) / (2.0 * PI * sample_period * cutoff_freq + 1.0);
        println!("Alpha = {alpha}");
        IIRFilter {
            last_sample_output: 0.0,
            cutoff_freq,
            sample_period,
            alpha,
        }
    }

    // sample is in range [0.0, 1.0]
    pub fn filter_sample(&mut self, input: f32) -> f32 {
        let output = self.last_sample_output + self.alpha * (input - self.last_sample_output);
        self.last_sample_output = output;
        output
    }
}
