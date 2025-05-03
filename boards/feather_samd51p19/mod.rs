#![no_std]

//! Pin aliases for the Feather-shaped SAMD51P19 board.

pub use anyhal::hal::gpio::{Pin, Port};

const fn pin(port: Port, index: u8) -> Pin {
    Pin::new(port, index)
}

pub const A0: Pin = pin(Port::B, 8);
pub const A1: Pin = pin(Port::B, 9);
pub const A2: Pin = pin(Port::A, 5);
pub const A3: Pin = pin(Port::A, 6);
pub const A4: Pin = pin(Port::A, 7);
pub const A5: Pin = pin(Port::A, 8);
pub const D0: Pin = pin(Port::B, 15);
pub const D1: Pin = pin(Port::D, 8);
pub const D4: Pin = pin(Port::D, 10);
pub const D5: Pin = pin(Port::B, 22);
pub const D6: Pin = pin(Port::B, 23);
pub const D9: Pin = pin(Port::B, 26);
pub const D10: Pin = pin(Port::B, 28);
pub const D11: Pin = pin(Port::B, 30);
pub const D12: Pin = pin(Port::C, 30);
pub const D13: Pin = pin(Port::B, 0);
pub const LED: Pin = D13;
pub const MISO: Pin = pin(Port::B, 14);
pub const MOSI: Pin = pin(Port::B, 12);
pub const RX: Pin = D0;
pub const SCK: Pin = pin(Port::B, 13);
pub const SCL: Pin = pin(Port::A, 17);
pub const SDA: Pin = pin(Port::A, 16);
pub const TX: Pin = D1;
