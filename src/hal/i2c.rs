//! Device-independent blocking I2C contracts.

use crate::hal::Result;

/// Blocking seven-bit-address I2C controller.
pub trait I2cBus {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<()>;
    fn read(&mut self, address: u8, bytes: &mut [u8]) -> Result<()>;

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<()> {
        self.write(address, write)?;
        self.read(address, read)
    }
}
