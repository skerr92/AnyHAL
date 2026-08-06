//! Device-independent single-channel PWM contract.

use crate::hal::Result;

/// A normalized PWM output.
///
/// Duty cycle spans the complete `u16` range: `0` is always low and
/// `u16::MAX` is always high, independent of the timer's native resolution.
pub trait PwmOutput {
    fn frequency_hz(&self) -> u32;
    fn duty_cycle(&self) -> u16;
    fn set_duty_cycle(&mut self, duty_cycle: u16) -> Result<()>;
    fn enable(&mut self) -> Result<()>;
    fn disable(&mut self) -> Result<()>;
    fn is_enabled(&self) -> bool;
}
