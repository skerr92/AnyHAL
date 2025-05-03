//! Device-independent polling serial-console contract.

use crate::hal::Result;

pub trait SerialConsole {
    /// Services transport events and returns whether useful work occurred.
    fn poll(&mut self) -> bool;

    /// Attempts a non-blocking write.
    fn write(&mut self, bytes: &[u8]) -> Result<usize>;

    /// Attempts a non-blocking read.
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize>;
}
