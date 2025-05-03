#![no_std]
#![allow(unsafe_code)]

//! STM32G431VBT6 chip support (LQFP100, 128 KiB flash, 32 KiB SRAM).
pub mod gpio;
pub mod runtime;
pub use gpio::{AlternatePin, InputPin, OutputPin};
pub const RESET_CORE_HZ: u32 = 16_000_000;
