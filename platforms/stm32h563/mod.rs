#![no_std]
#![allow(unsafe_code)]

//! STM32H563 chip support. Initial bring-up uses the 64 MHz reset HSI clock.

pub mod delay;
pub mod gpio;
pub mod runtime;

pub use delay::SysTickDelay;
pub use gpio::{AlternatePin, InputPin, OutputPin};

pub const RESET_CORE_HZ: u32 = 64_000_000;
