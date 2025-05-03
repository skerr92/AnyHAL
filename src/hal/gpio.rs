//! Portable digital GPIO contracts and pin identities.

use super::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Port {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    I = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pin {
    port: Port,
    index: u8,
}

impl Pin {
    pub const fn new(port: Port, index: u8) -> Self {
        assert!(index < 32, "GPIO pin index must be below 32");
        Self { port, index }
    }

    pub const fn port(self) -> Port {
        self.port
    }

    pub const fn index(self) -> u8 {
        self.index
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pull {
    None,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputType {
    PushPull,
    OpenDrain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Speed {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlternateFunction(u8);

impl AlternateFunction {
    pub const fn new(number: u8) -> Option<Self> {
        if number < 16 {
            Some(Self(number))
        } else {
            None
        }
    }

    pub const fn number(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlternateConfig {
    pub function: AlternateFunction,
    pub pull: Pull,
    pub output_type: OutputType,
    pub speed: Speed,
}

pub trait InputPin {
    fn is_high(&self) -> bool;

    fn is_low(&self) -> bool {
        !self.is_high()
    }
}

pub trait OutputPin {
    fn set_low(&mut self) -> Result<()>;
    fn set_high(&mut self) -> Result<()>;
    fn toggle(&mut self) -> Result<()>;
    fn is_set_high(&self) -> bool;
}
