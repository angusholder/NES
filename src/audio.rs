use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use log::{warn};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired};

pub struct APU {
    output_buffer: Option<SampleBuffer>,
    square_wave1: SquareWave,
    square_wave2: SquareWave,

    enable_square1: bool,
    enable_square2: bool,
    enable_triangle: bool,
    enable_noise: bool,
    enable_dmc: bool,

    sq1_samples: Vec<f32>,
    sq2_samples: Vec<f32>,
    mixed_samples: Vec<f32>,

    last_cpu_cycles: u64,
}

pub struct SampleBuffer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    samples_per_second: u32,
}

impl SampleBuffer {
    pub fn new(spec: &AudioSpec) -> SampleBuffer {
        SampleBuffer {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            samples_per_second: spec.freq as u32,
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
            *x = buffer.pop_front().unwrap_or(0.0);
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

pub fn create_audio_device(sdl: &sdl2::Sdl) -> AudioDevice<NesAudioCallback> {
    let audio_subsystem = sdl.audio().unwrap();
    let audio_spec = AudioSpecDesired {
        freq: Some(48_000),
        channels: Some(1),
        samples: None,
    };
    audio_subsystem.open_playback(None, &audio_spec, |spec: AudioSpec| {
        println!("Got audio spec: {spec:?}");
        NesAudioCallback {
            output_buffer: SampleBuffer::new(&spec),
        }
    }).unwrap()
}

impl APU {
    pub fn new() -> APU {
        APU {
            output_buffer: None,
            square_wave1: SquareWave::new(),
            square_wave2: SquareWave::new(),

            enable_square1: false,
            enable_square2: false,
            enable_triangle: false,
            enable_noise: false,
            enable_dmc: false,

            sq1_samples: Vec::new(),
            sq2_samples: Vec::new(),
            mixed_samples: Vec::new(),

            last_cpu_cycles: 0,
        }
    }

    pub fn attach_output_device(&mut self, device: &mut AudioDevice<NesAudioCallback>) {
        let mut sample_buffer = device.lock().get_output_buffer();
        sample_buffer.clear();
        self.output_buffer = Some(sample_buffer);
    }

    pub fn run_until_cycle(&mut self, end_cpu_cycle: u64) {
        let start_cpu_cycle = self.last_cpu_cycles;
        // If we have no output, don't bother generating any samples
        let samples_per_second = self.output_buffer.as_ref().map(|b| b.samples_per_second).unwrap_or(0);

        let start_time_s = start_cpu_cycle as f64 / CPU_FREQ as f64;
        let step_duration_s = (end_cpu_cycle - start_cpu_cycle) as f64 / CPU_FREQ as f64;
        let samples_to_output = (samples_per_second as f64 * step_duration_s) as usize;

        self.sq1_samples.resize(samples_to_output, 0f32);
        self.sq2_samples.resize(samples_to_output, 0f32);
        self.mixed_samples.resize(samples_to_output, 0f32);

        if self.enable_square1 {
            self.square_wave1.output_samples(start_time_s, step_duration_s, &mut self.sq1_samples);
        }
        if self.enable_square2 {
            self.square_wave2.output_samples(start_time_s, step_duration_s, &mut self.sq2_samples);
        }

        for ((s0, s1), out) in self.sq1_samples.iter().zip(self.sq2_samples.iter()).zip(self.mixed_samples.iter_mut()) {
            *out = *s0 + *s1;
        }

        if !self.mixed_samples.is_empty() {
            if let Some(output_buffer) = self.output_buffer.as_mut() {
                output_buffer.write_samples(&self.mixed_samples);
            }
        }

        self.last_cpu_cycles = end_cpu_cycle;
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

            0x4015 => {
                self.enable_square1 = (value & 0x01) != 0;
                self.enable_square2 = (value & 0x02) != 0;
                self.enable_triangle = (value & 0x04) != 0;
                self.enable_noise = (value & 0x08) != 0;
                self.enable_dmc = (value & 0x10) != 0;
            }

            _ => {}
        }
    }
}

const CPU_FREQ: u32 = 1_789_773; // 1.789773 MHz

pub struct NesAudioCallback {
    output_buffer: SampleBuffer,
}

impl NesAudioCallback {
    pub fn get_output_buffer(&self) -> SampleBuffer {
        self.output_buffer.clone_ref()
    }
}

impl AudioCallback for NesAudioCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.output_buffer.output_samples(out);
    }
}

struct SquareWave {
    phase: f32,
    volume: f32,

    duty_cycle: f32,
    period: u32,
}

impl SquareWave {
    fn new() -> SquareWave {
        SquareWave {
            phase: 0.0,
            volume: 0.1,
            duty_cycle: 0.5,
            period: 0, // Range: 0-0x7FF / 0-2047 / 12.428KHz-54Hz
        }
    }

    fn get_pulse_tone_hz(&self) -> f32 {
        CPU_FREQ as f32 / (16.0 * (self.period as f32 + 1.0))
    }

    fn output_samples(
        &mut self,
        step_start_time_s: f64,
        step_duration_s: f64,
        output: &mut [f32],
    ) {
        step_square_wave(
            step_start_time_s,
            step_duration_s,
            output,
            self.period,
            self.volume,
            self.duty_cycle,
        )
    }

    // $4003/$4007
    fn write_coarse_tune(&mut self, value: u8) {
        self.phase = 0.0;
        self.period = self.period & 0x00FF | ((value as u32 & 0x7) << 8);
        // TODO: Reset length counter
    }

    // $4002/$4006
    fn write_fine_tune(&mut self, value: u8) {
        self.period = self.period & 0xFF00 | value as u32;
    }

    // $4000/$4004
    fn write_control(&mut self, value: u8) {
        self.duty_cycle = match value >> 6 {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => unreachable!(),
        };
    }

    // $4001/$4005
    fn write_ramp(&mut self, _value: u8) {

    }
}

fn step_square_wave(
    step_start_time_s: f64,
    step_duration_s: f64,
    output: &mut [f32],
    apu_period: u32,
    volume: f32,
    duty_cycle: f32,
) {
    let period_s: f64 = (16 * (apu_period + 1)) as f64 / CPU_FREQ as f64;
    if apu_period < 8 {
        output.fill(0.0);
        // All zeroes
        return;
    }

    let time_step = step_duration_s / output.len() as f64;
    for (i, sample) in output.iter_mut().enumerate() {
        let now_s = step_start_time_s + time_step * i as f64;
        let phase = (now_s / period_s) % 1.0;
        if phase <= duty_cycle as f64 { // duty_cycle
            *sample = volume;
        } else {
            *sample = -volume;
        };
    }
}
