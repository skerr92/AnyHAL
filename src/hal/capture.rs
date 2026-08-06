//! Device-independent pulse capture contract.

use crate::hal::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PulseMeasurement {
    pub period_ticks: u32,
    pub high_ticks: u32,
}

impl PulseMeasurement {
    pub const fn period_us(self, tick_hz: u32) -> u32 {
        ticks_to_us(self.period_ticks, tick_hz)
    }

    pub const fn high_us(self, tick_hz: u32) -> u32 {
        ticks_to_us(self.high_ticks, tick_hz)
    }
}

pub trait PulseCapture {
    fn tick_hz(&self) -> u32;
    fn try_measure(&mut self) -> Result<Option<PulseMeasurement>>;
}

const fn ticks_to_us(ticks: u32, tick_hz: u32) -> u32 {
    if tick_hz == 0 {
        return 0;
    }
    ((ticks as u64 * 1_000_000 + tick_hz as u64 / 2) / tick_hz as u64) as u32
}
