use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LinearCounter {
    counter: u8,
    pub counter_reload_value: u8,
    pub reload_flag: bool,
    pub control_flag: bool,
}

impl LinearCounter {
    pub fn new() -> LinearCounter {
        LinearCounter {
            counter: 0,
            counter_reload_value: 0,
            reload_flag: false,
            control_flag: false,
        }
    }

    pub fn tick(&mut self) {
        // When the frame counter generates a linear counter clock, the following actions occur in order:

        // If the linear counter reload flag is set, the linear counter is reloaded with the counter reload value
        if self.reload_flag {
            self.counter = self.counter_reload_value;
            return;
        }
        // otherwise if the linear counter is non-zero, it is decremented
        else if self.counter > 0 {
            self.counter -= 1;
        }

        // If the control flag is clear, the linear counter reload flag is cleared.
        if !self.control_flag {
            self.reload_flag = false;
        }
    }

    pub fn is_zero(&self) -> bool {
        self.counter == 0
    }
}
