//! Device-independent blocking SPI contracts.

use crate::hal::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Polarity {
    IdleLow,
    IdleHigh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    CaptureOnFirstTransition,
    CaptureOnSecondTransition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitOrder {
    MostSignificantFirst,
    LeastSignificantFirst,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub frequency_hz: u32,
    pub polarity: Polarity,
    pub phase: Phase,
    pub bit_order: BitOrder,
}

impl Config {
    pub const fn mode0(frequency_hz: u32) -> Self {
        Self {
            frequency_hz,
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
            bit_order: BitOrder::MostSignificantFirst,
        }
    }
}

pub trait SpiBus {
    fn transfer_byte(&mut self, byte: u8) -> Result<u8>;

    fn transfer_in_place(&mut self, bytes: &mut [u8]) -> Result<()> {
        for byte in bytes {
            *byte = self.transfer_byte(*byte)?;
        }
        Ok(())
    }

    fn write(&mut self, bytes: &[u8]) -> Result<()> {
        for &byte in bytes {
            self.transfer_byte(byte)?;
        }
        Ok(())
    }
}
