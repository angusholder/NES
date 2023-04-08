pub struct LengthCounter {
    length: u8,
    pub halt: bool,
    channel_enabled: bool,
}

const LENGTH_LUT: [u8; 32] = [
    10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
];

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter {
            length: 0,
            halt: true,
            channel_enabled: true,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.length == 0
    }

    pub fn set_channel_enabled(&mut self, enabled: bool) {
        self.channel_enabled = enabled;
        if !enabled {
            self.length = 0;
        }
    }

    pub fn set_value(&mut self, value: u8) {
        if self.channel_enabled {
            let index = value as usize >> 3;
            self.length = LENGTH_LUT[index];
        } else {
            self.length = 0;
        }
    }

    pub fn tick(&mut self) {
        if !self.channel_enabled {
            return;
        }
        if self.halt {
            return;
        }

        if self.length > 0 {
            self.length -= 1;
        }
    }
}
