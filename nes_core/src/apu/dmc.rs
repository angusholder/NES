use std::rc::Rc;
use crate::mapper::Mapper;
use crate::nes::{InterruptSource, Signals};

pub struct DMC {
    irq_enabled: bool,
    loop_flag: bool,
    rate: u32, //
    timer: u32,

    // Output unit
    shift_register: u8,
    bits_remaining: u8,
    output_level: u8, // 0-127
    silence: bool,

    sample_address: u16,
    sample_length: u32,
    sample_buffer: Option<u8>,

    // Memory reader
    reader_address_buffer: u16,
    reader_bytes_remaining: u32,

    signals: Rc<Signals>,
    mapper: Mapper,
}

static DMC_RATE_PERIODS: [u32; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54
];

impl DMC {
    pub fn new(mapper: Mapper, signals: Rc<Signals>) -> DMC {
        let mut dmc = DMC {
            irq_enabled: false,
            loop_flag: false,
            rate: DMC_RATE_PERIODS[0],
            timer: 0,

            shift_register: 0,
            bits_remaining: 0,
            output_level: 0,
            silence: false,

            sample_address: 0xC000,
            sample_length: 0,
            sample_buffer: None,

            reader_address_buffer: 0x0000,
            reader_bytes_remaining: 0,

            signals,
            mapper,
        };
        dmc.start_new_output_cycle();
        dmc
    }

    pub fn tick(&mut self) {
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.clock_output_unit();
            self.timer = self.rate;
        }
    }

    // https://www.nesdev.org/wiki/APU_DMC#Output_unit
    fn clock_output_unit(&mut self) {
        if !self.silence {
            if self.shift_register & 1 == 1 {
                if self.output_level + 2 <= 127 {
                    self.output_level += 2;
                }
            } else {
                if self.output_level >= 2 {
                    self.output_level -= 2;
                }
            }
        }
        self.shift_register >>= 1;
        self.bits_remaining -= 1;

        if self.bits_remaining == 0 {
            self.start_new_output_cycle();
        }
    }

    // https://www.nesdev.org/wiki/APU_DMC#Output_unit
    fn start_new_output_cycle(&mut self) {
        self.bits_remaining = 8;
        if let Some(sample) = self.sample_buffer.take() {
            self.silence = false;
            self.shift_register = sample;
            if self.reader_bytes_remaining > 0 {
                self.perform_memory_read();
            }
        } else {
            self.silence = true;
        }
    }

    // https://www.nesdev.org/wiki/APU_DMC#Memory_reader
    fn perform_memory_read(&mut self) {
        if self.reader_bytes_remaining == 0 {
            return;
        }

        // TODO: Stall CPU

        self.sample_buffer = Some(self.mapper.read_main_bus(self.reader_address_buffer));

        if self.reader_address_buffer < 0xFFFF {
            self.reader_address_buffer += 1;
        } else {
            // Wrap back around to the bottom address.
            self.reader_address_buffer = 0x8000;
        }

        self.reader_bytes_remaining -= 1;
        if self.reader_bytes_remaining == 0 {
            if self.loop_flag {
                self.restart_sample();
            } else if self.irq_enabled {
                self.signals.request_interrupt(InterruptSource::APU_DMC);
            }
        }
    }

    fn restart_sample(&mut self) {
        self.reader_address_buffer = self.sample_address;
        self.reader_bytes_remaining = self.sample_length;
    }

    pub fn get_current_output(&self) -> u8 {
        self.output_level
    }

    pub fn set_channel_enabled(&mut self, enabled: bool) {
        if !enabled {
            // If the DMC bit is clear, the DMC bytes remaining will be set to 0 and the DMC will silence when it empties.
            self.reader_bytes_remaining = 0;
        } else {
            // If the DMC bit is set, the DMC sample will be restarted only if its bytes remaining is 0.
            // If there are bits remaining in the 1-byte sample buffer, these will finish playing before the next sample is fetched.
            if self.reader_bytes_remaining == 0 {
                self.restart_sample();
            }
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
