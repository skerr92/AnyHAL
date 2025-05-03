#![no_std]
#![allow(unsafe_code)]

//! ATSAMD51P19A-specific HAL implementation and runtime support.

pub mod clock;
pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod runtime;
pub mod spi;
pub mod usb_serial;

pub use clock::{configure_dfll48m, Clocks};
pub use delay::SysTickDelay;
pub use gpio::{AlternatePin, InputPin, OutputPin};
pub use i2c::Sercom1I2c;
pub use spi::Sercom4Spi;
pub use usb_serial::UsbCdcConsole;
