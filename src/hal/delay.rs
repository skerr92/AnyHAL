//! Portable blocking delay contracts.

use super::Result;

pub trait DelayMs {
    fn delay_ms(&mut self, milliseconds: u32) -> Result<()>;
}
