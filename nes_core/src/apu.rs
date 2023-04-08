use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use bitflags::bitflags;
use log::{info, warn};

mod square;
mod triangle;
mod noise;
mod envelope;
mod sweep;
mod divider;
mod length_counter;
mod linear_counter;

use crate::apu::noise::Noise;
use crate::apu::square::{SquareUnit, SquareWave};
use crate::apu::triangle::TriangleWave;
use crate::mapper;
use crate::nes::{IRQSource, Signals};

pub struct APU {
    output_buffer: Option<SampleBuffer>,

    square_wave1: SquareWave,
    square_wave2: SquareWave,
    triangle_wave: TriangleWave,
    noise: Noise,

    /// Which channels the game wants enabled currently.
    guest_enabled_channels: AudioChannels,
    /// The user can override to mute a channel that the game has enabled.
    host_enabled_channels: AudioChannels,

    irq_inhibit: bool,
    frame_counter_mode: FrameCountMode,

    sq1_samples: Vec<u8>,
    sq2_samples: Vec<u8>,
    tri_samples: Vec<u8>,
    noise_samples: Vec<u8>,
    dmc_samples: Vec<u8>,
    mixed_samples: Vec<f32>,

    last_cpu_cycles: u64,
    apu_cycle: u64,
    signals: Rc<Signals>,
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

pub struct SampleBuffer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    samples_per_second: u32,
}

impl SampleBuffer {
    pub fn new(freq: u32) -> SampleBuffer {
        SampleBuffer {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            samples_per_second: freq,
        }
    }

    pub fn clone_ref(&self) -> SampleBuffer {
        SampleBuffer {
            buffer: self.buffer.clone(),
            samples_per_second: self.samples_per_second,
        }
    }

    pub fn output_samples(&mut self, out: &mut [f32]) {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() < out.len() {
            warn!("Not enough samples in buffer - needed {}, got {}", out.len(), buffer.len());
        }
        for x in out.iter_mut() {
            *x = buffer.pop_front().unwrap_or(-1.0);
        }
    }

    pub fn write_samples(&mut self, samples: &[f32]) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend(samples);
    }

    pub fn clear(&mut self) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clear();
    }
}

impl APU {
    pub fn new(signals: Rc<Signals>) -> APU {
        APU {
            output_buffer: None,

            square_wave1: SquareWave::new(SquareUnit::Pulse1),
            square_wave2: SquareWave::new(SquareUnit::Pulse2),
            triangle_wave: TriangleWave::new(),
            noise: Noise::new(),

            guest_enabled_channels: AudioChannels::empty(),
            host_enabled_channels: AudioChannels::all(),

            irq_inhibit: false,
            frame_counter_mode: FrameCountMode::Step4,

            sq1_samples: Vec::new(),
            sq2_samples: Vec::new(),
            tri_samples: Vec::new(),
            noise_samples: Vec::new(),
            dmc_samples: Vec::new(),
            mixed_samples: Vec::new(),

            last_cpu_cycles: 0,
            apu_cycle: 0,
            signals,
        }
    }

    pub fn attach_output_device(&mut self, output_buffer: SampleBuffer) {
        self.output_buffer = Some(output_buffer);
    }

    pub fn step_cycle(&mut self, cpu_cycle: u64) {
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
            _ => {
                // Nothing changed, don't call run_until_cycle
                return;
            }
        }
        self.run_until_cycle(cpu_cycle);
    }

    fn tick_envelope_and_triangle(&mut self) {
        self.square_wave1.envelope.tick();
        self.square_wave2.envelope.tick();
        self.triangle_wave.linear_counter.tick();
    }

    fn tick_length_counters_and_sweep(&mut self) {
        self.square_wave1.tick_length_and_swap();
        self.square_wave2.tick_length_and_swap();
        self.triangle_wave.length_counter.tick();
    }

    fn trigger_irq(&mut self) {
        if self.irq_inhibit {
            return;
        }
        self.signals.request_irq(IRQSource::APU_FRAME_COUNTER);
    }

    pub fn run_until_cycle(&mut self, end_cpu_cycle: u64) {
        let start_cpu_cycle = self.last_cpu_cycles;
        // If we have no output, don't bother generating any samples
        let samples_per_second = self.output_buffer.as_ref().map(|b| b.samples_per_second).unwrap_or(0);

        let start_time_s = start_cpu_cycle as f64 / CPU_FREQ as f64;
        let step_duration_s = (end_cpu_cycle - start_cpu_cycle) as f64 / CPU_FREQ as f64;
        let samples_to_output = (samples_per_second as f64 * step_duration_s) as usize;

        self.sq1_samples.resize(samples_to_output, 0);
        self.sq2_samples.resize(samples_to_output, 0);
        self.tri_samples.resize(samples_to_output, 0);
        self.noise_samples.resize(samples_to_output, 0);
        self.dmc_samples.resize(samples_to_output, 0);
        self.mixed_samples.resize(samples_to_output, 0f32);
        self.mixed_samples.fill(0.0);

        if self.channel_enabled(AudioChannels::SQUARE1) {
            self.square_wave1.output_samples(start_time_s, step_duration_s, &mut self.sq1_samples);
        }
        if self.channel_enabled(AudioChannels::SQUARE2) {
            self.square_wave2.output_samples(start_time_s, step_duration_s, &mut self.sq2_samples);
        }
        if self.channel_enabled(AudioChannels::TRIANGLE) {
            self.triangle_wave.output_samples(start_time_s, step_duration_s, &mut self.tri_samples);
        }
        if self.channel_enabled(AudioChannels::NOISE) {
            self.noise.output_samples(samples_per_second, &mut self.noise_samples);
        }

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

        for i in 0..samples_to_output {
            // Mixing formula from here: https://www.nesdev.org/wiki/APU_Mixer
            let pulse1: u8 = self.sq1_samples[i] & 15; // 0 to 15 (4-bit)
            let pulse2: u8 = self.sq2_samples[i] & 15; // 0 to 15 (4-bit)
            let triangle: u8 = self.tri_samples[i] & 15; // 0 to 15 (4-bit)
            let noise: u8 = self.noise_samples[i] & 15; // 0 to 15 (4-bit)
            let dmc: u8 = self.dmc_samples[i] & 127; // 0 to 127 (7-bit)

            let pulse_out = PULSE_OUT[(pulse1 + pulse2) as usize];
            let tnd_out = TND_OUT[(3 * triangle + 2 * noise + dmc) as usize];
            let output = pulse_out + tnd_out;
            self.mixed_samples[i] = output * 2.0 - 1.0;
        }

        if !self.mixed_samples.is_empty() {
            if let Some(output_buffer) = self.output_buffer.as_mut() {
                output_buffer.write_samples(&self.mixed_samples);
            }
        }

        self.last_cpu_cycles = end_cpu_cycle;
    }

    pub fn read_register(&mut self, addr: u16, cpu_cycle: u64) -> u8 {
        match addr {
            0x4015 => {
                self.read_status_register(cpu_cycle)
            }
            _ => {
                mapper::out_of_bounds_read("APU", addr)
            }
        }
    }

    fn read_status_register(&mut self, cpu_cycle: u64) -> u8 {
        // https://www.nesdev.org/wiki/APU#Status_($4015)
        // TODO: Implement the rest of $4015 APU Status register

        self.run_until_cycle(cpu_cycle);

        let mut status = 0u8;

        if self.signals.is_active(IRQSource::APU_FRAME_COUNTER) {
            status |= 0x40;
            self.signals.acknowledge_irq(IRQSource::APU_FRAME_COUNTER);
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

        status
    }

    pub fn write_register(&mut self, addr: u16, value: u8, cpu_cycle: u64) {
        self.run_until_cycle(cpu_cycle);

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

            0x4015 => {
                self.guest_enabled_channels = AudioChannels::from_bits_truncate(value);
                self.square_wave1.set_enabled(self.guest_enabled_channels.contains(AudioChannels::SQUARE1));
                self.square_wave2.set_enabled(self.guest_enabled_channels.contains(AudioChannels::SQUARE2));
                self.triangle_wave.set_enabled(self.guest_enabled_channels.contains(AudioChannels::TRIANGLE));
            }
            0x4017 => {
                self.write_frame_counter(value)
            }

            _ => {}
        }
    }

    fn write_frame_counter(&mut self, value: u8) {
        self.irq_inhibit = value & 0x40 != 0;
        self.frame_counter_mode = if value & 0x80 == 0 { FrameCountMode::Step4 } else { FrameCountMode::Step5 };
        self.apu_cycle = 0;
        info!("IRQ inhibit = {}, frame counter mode = {:?}", self.irq_inhibit, self.frame_counter_mode);
    }

    fn channel_enabled(&self, channel: AudioChannels) -> bool {
        let enabled = self.host_enabled_channels & self.guest_enabled_channels;
        enabled.contains(channel)
    }

    pub fn toggle_channel(&mut self, channel: AudioChannels) {
        self.host_enabled_channels.toggle(channel);
        let state = if self.host_enabled_channels.contains(channel) { "on" } else { "off" };
        info!("Toggled channel {channel:?} to {state}")
    }
}

const CPU_FREQ: u32 = 1_789_773; // 1.789773 MHz
