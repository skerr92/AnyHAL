#![no_std]
#![allow(unsafe_code)]

//! ATSAMD51P19A-specific HAL implementation and runtime support.

pub mod adc;
pub mod capture;
pub mod clock;
pub mod dac;
pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod pwm;
mod routes;
pub mod runtime;
pub mod spi;
pub mod usb_serial;

pub use adc::Adc0Input;
pub use capture::Tc5PulseCapture;
pub use clock::{configure_dfll48m, Clocks};
pub use dac::Dac1Output;
pub use delay::SysTickDelay;
pub use gpio::{AlternatePin, InputPin, OutputPin};
pub use i2c::Sercom1I2c;
pub use pwm::Tc4Pwm;
pub use spi::Sercom4Spi;
pub use usb_serial::UsbCdcConsole;

/// Supplier package selected when this platform crate was built.
pub const PACKAGE: &str = routes::PACKAGE;
