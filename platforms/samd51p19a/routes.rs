//! Package-aware SAMD51P19A pin capability lookup.
//!
//! Route functions are generated at build time from the checked-in export of
//! Microchip's SAMD51 device pack. Platform drivers never embed board wiring.

use anyhal::hal::gpio::{Pin, Port};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AnalogRoute {
    pub channel: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TimerRoute {
    pub waveform_output: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EicRoute {
    pub extint: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SercomRoute {
    pub pad: u8,
    pub function: u8,
    pub ioset: u8,
}

include!(concat!(env!("OUT_DIR"), "/routes_generated.rs"));
