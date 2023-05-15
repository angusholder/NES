pub struct DMC {
    irq_enabled: bool,
    loop_flag: bool,
    rate: u32, //

    output_level: u8, // 0-127

    sample_address: u16,
    sample_length: u32,
    sample_buffer: u8,

    reader_address_buffer: u16,
    reader_bytes_remaining: u32,
}

static DMC_RATE_PERIODS: [u32; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54
];

impl DMC {
    pub fn new() -> DMC {
        DMC {
            irq_enabled: false,
            loop_flag: false,
            rate: DMC_RATE_PERIODS[0],

            output_level: 0,

            sample_address: 0xC000,
            sample_length: 0,
            sample_buffer: 0x00,

            reader_address_buffer: 0x0000,
            reader_bytes_remaining: 0,
        }
    }

    pub fn tick(&mut self) {
        // TODO: Implement delta DMC
    }

    pub fn get_current_output(&self) -> u8 {
        self.output_level
    }

    pub fn set_channel_enabled(&mut self, enabled: bool) {
        if !enabled {
            // If the DMC bit is clear, the DMC bytes remaining will be set to 0 and the DMC will silence when it empties.
            self.reader_bytes_remaining = 0;
            // TODO: Do we need to do anything else here?
        } else {
            // If the DMC bit is set, the DMC sample will be restarted only if its bytes remaining is 0.
            // If there are bits remaining in the 1-byte sample buffer, these will finish playing before the next sample is fetched.
            // TODO
        }
    }

    pub fn has_bytes_remaining(&self) -> bool {
        self.reader_bytes_remaining > 0
    }

    pub fn write_control(&mut self, value: u8) {
        self.irq_enabled = value & 0x80 != 0;
        self.loop_flag = value & 0x40 != 0;
        let rate_index = (value & 0xF) as usize;
        self.rate = DMC_RATE_PERIODS[rate_index];
    }

    pub fn write_direct_load(&mut self, value: u8) {
        self.output_level = value & 0b111_1111;
    }

    // DMC samples are from the address range $C000-$FFFF
    pub fn write_sample_address(&mut self, value: u8) {
        self.sample_address = 0xC000 | ((value as u16) << 6);
    }

    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length = (value as u32) * 16 + 1;
    }
}
