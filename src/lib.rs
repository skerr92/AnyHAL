#![cfg_attr(not(feature = "host"), no_std)]
#![deny(unsafe_code)]

//! Portable HAL contracts shared by independently packaged platforms and boards.

pub const VERSION: (u8, u8, u8) = (0, 1, 0);

pub mod hal;
pub use hal::{Error, Result};

#[cfg(feature = "host")]
pub mod testing;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_package() {
        assert_eq!(VERSION, (0, 1, 0));
    }
}
