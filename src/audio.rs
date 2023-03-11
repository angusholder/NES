use std::cmp::{max, min};
use log::info;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired};

pub struct Audio {
    square_wave1: AudioDevice<SquareWave>,
    lut_index: usize,
}

impl Audio {
    pub fn new(audio_subsystem: &sdl2::AudioSubsystem) -> Audio {
        let audio_spec = AudioSpecDesired {
            freq: Some(48_000),
            channels: Some(1),
            samples: None,
        };
        let audio_out: AudioDevice<SquareWave> = audio_subsystem.open_playback(None, &audio_spec, |spec: AudioSpec| {
            println!("Got audio spec: {spec:?}");
            SquareWave::new(&spec)
        }).unwrap();
        Audio {
            square_wave1: audio_out,
            lut_index: 10,
        }
    }

    pub fn play(&mut self) {
        self.square_wave1.resume();
    }

    pub fn adjust_frequency(&mut self, delta: i32) {
        let mut generator = self.square_wave1.lock();
        self.lut_index = max(min(self.lut_index as i32 + delta, (PULSE_FREQ_PERIODS_LUT.len() - 1) as i32), 0) as usize;
        let (period, name) = PULSE_FREQ_PERIODS_LUT[self.lut_index];
        generator.period = period;
        info!("Period {period}, note = {name}, freq = {:.0}Hz", generator.get_pulse_tone_hz())
    }

    pub fn adjust_duty_cycle(&mut self, new_duty_cycle: f32) {
        info!("Setting duty cycle to {}", new_duty_cycle);
        self.square_wave1.lock().duty_cycle = new_duty_cycle;
    }
}

const CPU_FREQ: u32 = 1_789_773; // 1.789773 MHz

struct SquareWave {
    samples_per_second: f32,
    phase: f32,
    volume: f32,

    duty_cycle: f32,
    period: u32,

    samples_output: u64,
}

impl SquareWave {
    fn new(spec: &AudioSpec) -> SquareWave {
        SquareWave {
            samples_per_second: spec.freq as f32,
            phase: 0.0,
            volume: 0.1,
            duty_cycle: 0.5,
            period: PULSE_FREQ_PERIODS_LUT[10].0, // Range: 0-0x7FF / 0-2047 / 12.428KHz-54Hz
            samples_output: 0,
        }
    }

    fn get_pulse_tone_hz(&self) -> f32 {
        CPU_FREQ as f32 / (16.0 * (self.period as f32 + 1.0))
    }
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let time_secs = self.samples_output as f64 / self.samples_per_second as f64;
        let cpu_start_cycle = (time_secs * CPU_FREQ as f64) as u64;
        // We're simulating a whole second, then throwing some away. Could be better...
        let cpu_end_cycle = cpu_start_cycle + 1*CPU_FREQ as u64;
        let samples: Vec<f32> = step_square_wave(
            cpu_start_cycle,
            cpu_end_cycle,
            self.samples_per_second as u32,
            self.period,
            self.volume,
            self.duty_cycle,
        );
        out.copy_from_slice(&samples[..out.len()]);
        self.samples_output += out.len() as u64;
    }
}

fn step_square_wave(
    cpu_start_cycle: u64,
    cpu_end_cycle: u64,
    samples_per_second: u32,
    apu_period: u32,
    volume: f32,
    duty_cycle: f32,
) -> Vec<f32> {
    let period_s: f64 = (16 * (apu_period + 1)) as f64 / CPU_FREQ as f64;
    let start_time_s = cpu_start_cycle as f64 / CPU_FREQ as f64;
    let total_duration_s = (cpu_end_cycle - cpu_start_cycle) as f64 / CPU_FREQ as f64;
    let samples_to_output = (samples_per_second as f64 * total_duration_s) as usize;
    let mut output: Vec<f32> = vec![0.0; samples_to_output];
    if apu_period < 8 {
        // All zeroes
        return output;
    }

    let time_step = total_duration_s / samples_to_output as f64;
    for (i, sample) in output.iter_mut().enumerate() {
        let now_s = start_time_s + time_step * i as f64;
        let phase = (now_s / period_s) % 1.0;
        if phase <= duty_cycle as f64 { // duty_cycle
            *sample = volume;
        } else {
            *sample = -volume;
        };
    }

    output
}

static PULSE_FREQ_PERIODS_LUT: [(u32, &str); 75] = [
    (0x07F0, "A-1"),
    (0x077C, "Bb1"),
    (0x0710, "B-1"),
    (0x06AC, "C-2"),
    (0x064C, "C#2"),
    (0x05F2, "D-2"),
    (0x059E, "Eb2"),
    (0x054C, "E-2"),
    (0x0501, "F-2"),
    (0x04B8, "F#2"),
    (0x0474, "G-2"),
    (0x0434, "Ab2"),
    (0x03F8, "A-2"),
    (0x03BE, "Bb2"),
    (0x0388, "B-2"),
    (0x0356, "C-3"),
    (0x0326, "C#3"),
    (0x02F9, "D-3"),
    (0x02CF, "Eb3"),
    (0x02A6, "E-3"),
    (0x0280, "F-3"),
    (0x025C, "F#3"),
    (0x023A, "G-3"),
    (0x021A, "Ab3"),
    (0x01FC, "A-3"),
    (0x01DF, "Bb3"),
    (0x01C4, "B-3"),
    (0x01AB, "C-4"),
    (0x0193, "C#4"),
    (0x017C, "D-4"),
    (0x0167, "Eb4"),
    (0x0153, "E-4"),
    (0x0140, "F-4"),
    (0x012E, "F#4"),
    (0x011D, "G-4"),
    (0x010D, "Ab4"),
    (0x00FE, "A-4"),
    (0x00EF, "Bb4"),
    (0x00E2, "B-4"),
    (0x00D5, "C-5"),
    (0x00C9, "C#5"),
    (0x00BE, "D-5"),
    (0x00B3, "Eb5"),
    (0x00A9, "E-5"),
    (0x00A0, "F-5"),
    (0x0097, "F#5"),
    (0x008E, "G-5"),
    (0x0086, "Ab5"),
    (0x007E, "A-5"),
    (0x0077, "Bb5"),
    (0x0071, "B-5"),
    (0x006A, "C-6"),
    (0x0064, "C#6"),
    (0x005F, "D-6"),
    (0x0059, "Eb6"),
    (0x0054, "E-6"),
    (0x0050, "F-6"),
    (0x004B, "F#6"),
    (0x0047, "G-6"),
    (0x0043, "Ab6"),
    (0x003F, "A-6"),
    (0x003B, "Bb6"),
    (0x0038, "B-6"),
    (0x0035, "C-7"),
    (0x0032, "C#7"),
    (0x002F, "D-7"),
    (0x002C, "Eb7"),
    (0x002A, "E-7"),
    (0x0028, "F-7"),
    (0x0026, "F#7"),
    (0x0024, "G-7"),
    (0x0022, "Ab7"),
    (0x0020, "A-7"),
    (0x001E, "Bb7"),
    (0x001C, "B-7"),
];
