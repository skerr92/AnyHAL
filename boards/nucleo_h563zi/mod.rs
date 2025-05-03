#![no_std]

//! NUCLEO-H563ZI board aliases (default solder-bridge configuration).
pub use anyhal::hal::gpio::{Pin, Port};
pub const LED_GREEN: Pin = Pin::new(Port::B, 0);
pub const LED_YELLOW: Pin = Pin::new(Port::F, 4);
pub const LED_RED: Pin = Pin::new(Port::G, 4);
pub const LED: Pin = LED_GREEN;
pub const USER_BUTTON: Pin = Pin::new(Port::C, 13);
pub const VCP_TX: Pin = Pin::new(Port::D, 8);
pub const VCP_RX: Pin = Pin::new(Port::D, 9);
