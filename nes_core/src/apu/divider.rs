pub struct Divider {
    counter: u8,
}

impl Divider {
    pub fn new(value: u8) -> Divider {
        Divider {
            counter: value,
        }
    }

    /// https://www.nesdev.org/wiki/APU#Glossary
    /// A divider outputs a clock periodically. It contains a period reload value, P, and a counter,
    /// that starts at P. When the divider is clocked, if the counter is currently 0, it is reloaded
    /// with P and generates an output clock, otherwise the counter is decremented. In other words,
    /// the divider's period is P + 1.
    pub fn tick(&mut self, reload_value: u8) -> bool {
        if self.counter == 0 {
            self.counter = reload_value;
            return true;
        } else {
            self.counter -= 1;
            return false;
        }
    }

    pub fn reset(&mut self, value: u8) {
        self.counter = value;
    }
}
