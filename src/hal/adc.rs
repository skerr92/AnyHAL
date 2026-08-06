//! Device-independent blocking analog-to-digital conversion.

use crate::hal::Result;

/// A claimed analog input sampled by a single ADC instance.
pub trait OneShotAdc {
    fn resolution_bits(&self) -> u8;
    fn reference_mv(&self) -> u32;
    fn read(&mut self) -> Result<u16>;

    fn read_mv(&mut self) -> Result<u32> {
        let sample = u32::from(self.read()?);
        let max = (1_u32 << self.resolution_bits()) - 1;
        Ok(sample * self.reference_mv() / max)
    }
}
