/// https://www.nesdev.org/wiki/APU_Noise
pub struct Noise {
    period: u32,
    feedback_bit_6: bool,
    shift_register: u16, // 15 bits
    volume: u8, // 0 to 15 (4-bit)
    constant_volume: bool,
    length_counter_halt: bool,
}

impl Noise {
    const PERIOD_LUT: [u32; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

    pub fn new() -> Noise {
        Noise {
            period: Self::PERIOD_LUT[0],
            feedback_bit_6: false,
            shift_register: 1,
            volume: 15,
            constant_volume: true,
            length_counter_halt: true,
        }
    }

    pub fn write_control(&mut self, value: u8) {
        self.volume = value & 0xF;
        self.constant_volume = value & 0x10 != 0;
        self.length_counter_halt = value & 0x20 != 0;
    }

    pub fn write_noise_freq1(&mut self, value: u8) {
        let period_index = (value & 0xF) as usize;
        self.period = Self::PERIOD_LUT[period_index];
        self.feedback_bit_6 = value & 0x80 != 0; // otherwise bit 1
    }

    pub fn write_noise_freq2(&mut self, _value: u8) {

    }

    pub fn output_samples(
        &mut self,
        _step_start_time_s: f64,
        _step_duration_s: f64,
        _output: &mut [u8],
    ) {

    }

    fn do_feedback(&mut self) {
        let mut sr = self.shift_register;

        let shift_amt = if self.feedback_bit_6 { 6 } else { 1 };
        let feedback = (sr & 1) ^ (sr >> shift_amt & 1);

        sr >>= 1;

        sr |= feedback << 14;

        self.shift_register = sr;
    }
}