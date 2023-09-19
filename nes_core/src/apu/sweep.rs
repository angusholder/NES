use serde::{Deserialize, Serialize};
use crate::apu::divider::Divider;

/// https://www.nesdev.org/wiki/APU_Sweep
#[derive(Serialize, Deserialize)]
pub struct Sweep {
    // Params (EPPP.NSSS):
    pub enabled: bool,
    pub divider_period: u8, // 1-8
    pub negate: bool,
    pub shift_count: u8, // 0-7

    // State:
    divider: Divider,
    reload_flag: bool,

    /// Whether to negate the change amount using two's complement or one's complement.
    /// See https://www.nesdev.org/wiki/APU_Sweep#Calculating_the_target_period
    ones_complement: bool,
}

impl Sweep {
    pub fn new(ones_complement: bool) -> Sweep {
        Sweep {
            ones_complement,

            enabled: false,
            divider_period: 1,
            negate: false,
            shift_count: 0,

            divider: Divider::new(0),
            reload_flag: true,
        }
    }

    pub fn tick(&mut self, period: &mut u32) {
        // When the frame counter sends a half-frame clock (at 120 or 96 Hz), two things happen

        // If the divider's counter is zero or the reload flag is true: The divider counter is set to P and the reload flag is cleared. Otherwise, the divider counter is decremented.
        if self.reload_flag {
            self.divider.reset(self.divider_period);
            self.reload_flag = false;
            return;
        }

        // If the divider's counter is zero, the sweep is enabled, and the sweep unit is not muting the channel: The pulse's period is set to the target period.
        if self.divider.tick(self.divider_period) {
            if self.enabled && !self.should_mute(*period) {
                *period = self.calculate_target_period(*period);
            }
        }
    }

    // $4001/$4005
    pub fn set_reload_flag(&mut self) {
        self.reload_flag = true;
    }

    pub fn should_mute(&self, current_period: u32) -> bool {
        let target_period = self.calculate_target_period(current_period);
        current_period < 8 || target_period > 0x7FF
    }

    pub fn calculate_target_period(&self, current_period: u32) -> u32 {
        let mut change_amount = (current_period >> self.shift_count) as i32;
        if self.negate {
            if self.ones_complement {
                change_amount = -change_amount - 1;
            } else {
                change_amount = -change_amount;
            }
        }

        current_period.saturating_add_signed(change_amount)
    }
}
