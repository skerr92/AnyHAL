//! Device-independent HAL contracts and types.

pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod serial;
pub mod spi;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidArgument,
    Unsupported,
    Platform,
    NoAcknowledge,
    Bus,
    WouldBlock,
}

pub type Result<T> = core::result::Result<T, Error>;
