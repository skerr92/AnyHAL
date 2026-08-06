//! Device-independent digital-to-analog output.

use crate::hal::Result;

/// A single voltage-output DAC channel.
pub trait AnalogOutput {
    fn resolution_bits(&self) -> u8;
    fn reference_mv(&self) -> u32;
    fn value(&self) -> u16;
    fn set_value(&mut self, value: u16) -> Result<()>;

    fn set_mv(&mut self, millivolts: u32) -> Result<()> {
        let maximum = (1_u32 << self.resolution_bits()) - 1;
        let clamped = millivolts.min(self.reference_mv());
        self.set_value((clamped * maximum / self.reference_mv()) as u16)
    }
}
