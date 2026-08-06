//! Blocking SERCOM1 I2C controller.

use crate::{routes, AlternatePin, Clocks};
use anyhal::hal::{
    gpio::{AlternateConfig, AlternateFunction, OutputType, Pin, Pull, Speed},
    i2c::I2cBus,
    Error, Result,
};

const SYNC_TIMEOUT: u32 = 1_000_000;
const SERCOM1_CORE_GCLK_ID: usize = 8;

pub struct Sercom1I2c {
    _sda: AlternatePin,
    _scl: AlternatePin,
}

impl Sercom1I2c {
    /// Claims SERCOM1 as an I2C controller on PA16 (SDA) and PA17 (SCL).
    ///
    /// External pull-up resistors are required. The caller must exclusively own
    /// SERCOM1, both pins, and their clock channel.
    ///
    /// # Safety
    ///
    /// The caller must ensure exclusive ownership of SERCOM1, the supplied
    /// pins, their PORT mux state, and the SERCOM1 generic-clock channel.
    pub unsafe fn claim(sda: Pin, scl: Pin, clocks: Clocks, frequency_hz: u32) -> Result<Self> {
        let sda_route = routes::sercom1(sda).ok_or(Error::InvalidArgument)?;
        let scl_route = routes::sercom1(scl).ok_or(Error::InvalidArgument)?;
        if sda_route.pad != 0
            || scl_route.pad != 1
            || sda_route.function != scl_route.function
            || sda_route.ioset != scl_route.ioset
        {
            return Err(Error::InvalidArgument);
        }
        let divider = clocks
            .core_hz()
            .checked_div(frequency_hz.checked_mul(2).ok_or(Error::InvalidArgument)?)
            .and_then(|value| value.checked_sub(5))
            .ok_or(Error::InvalidArgument)?;
        if divider == 0 || divider > u8::MAX.into() {
            return Err(Error::InvalidArgument);
        }

        let pin_config = AlternateConfig {
            function: AlternateFunction::new(sda_route.function).ok_or(Error::Platform)?,
            pull: Pull::None,
            output_type: OutputType::PushPull,
            speed: Speed::VeryHigh,
        };
        let sda = unsafe { AlternatePin::claim(sda, pin_config)? };
        let scl = unsafe { AlternatePin::claim(scl, pin_config)? };

        let gclk = unsafe { &*atsamd51p19a::Gclk::ptr() };
        let mclk = unsafe { &*atsamd51p19a::Mclk::ptr() };
        let i2c = unsafe { &*atsamd51p19a::Sercom1::ptr() }.i2cm();

        mclk.apbamask().modify(|_, w| w.sercom1_().set_bit());
        gclk.pchctrl(SERCOM1_CORE_GCLK_ID)
            .write(|w| w.gen().gclk0().chen().set_bit());

        i2c.ctrla().write(|w| w.swrst().set_bit());
        wait_for(|| !i2c.syncbusy().read().swrst().bit())?;
        i2c.ctrla().write(|w| w.mode().i2c_master());
        i2c.baud().write(|w| w.baud().bits(divider as u8));
        i2c.ctrla().modify(|_, w| w.enable().set_bit());
        wait_for(|| !i2c.syncbusy().read().enable().bit())?;
        i2c.status().write(|w| unsafe { w.busstate().bits(1) });
        wait_for(|| !i2c.syncbusy().read().sysop().bit())?;

        Ok(Self {
            _sda: sda,
            _scl: scl,
        })
    }

    fn registers(&self) -> &atsamd51p19a::sercom0::I2cm {
        unsafe { &*atsamd51p19a::Sercom1::ptr() }.i2cm()
    }

    fn start(&self, address: u8, read: bool) -> Result<()> {
        if address >= 0x80 {
            return Err(Error::InvalidArgument);
        }
        let i2c = self.registers();
        let address_byte = u16::from((address << 1) | u8::from(read));
        i2c.addr().write(|w| unsafe { w.addr().bits(address_byte) });
        self.wait_flag(if read { 2 } else { 1 })?;
        self.check_status()
    }

    fn wait_flag(&self, mask: u8) -> Result<()> {
        let i2c = self.registers();
        wait_for(|| i2c.intflag().read().bits() & mask != 0)
    }

    fn check_status(&self) -> Result<()> {
        let status = self.registers().status().read();
        if status.buserr().bit() || status.arblost().bit() {
            Err(Error::Bus)
        } else if status.rxnack().bit() {
            Err(Error::NoAcknowledge)
        } else {
            Ok(())
        }
    }

    fn stop(&self) {
        self.registers()
            .ctrlb()
            .modify(|_, w| unsafe { w.ackact().set_bit().cmd().bits(3) });
    }

    fn write_without_stop(&self, address: u8, bytes: &[u8]) -> Result<()> {
        self.start(address, false)?;
        let i2c = self.registers();
        for &byte in bytes {
            i2c.data()
                .write(|w| unsafe { w.data().bits(u32::from(byte)) });
            self.wait_flag(1)?;
            self.check_status()?;
        }
        Ok(())
    }

    fn read_after_start(&self, address: u8, bytes: &mut [u8]) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        self.start(address, true)?;
        let i2c = self.registers();
        let last = bytes.len() - 1;
        for (index, byte) in bytes.iter_mut().enumerate() {
            if index == last {
                i2c.ctrlb().modify(|_, w| w.ackact().set_bit());
            }
            *byte = i2c.data().read().data().bits() as u8;
            if index == last {
                self.stop();
            } else {
                i2c.ctrlb()
                    .modify(|_, w| unsafe { w.ackact().clear_bit().cmd().bits(2) });
                self.wait_flag(2)?;
                self.check_status()?;
            }
        }
        Ok(())
    }
}

impl I2cBus for Sercom1I2c {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<()> {
        let result = self.write_without_stop(address, bytes);
        self.stop();
        result
    }

    fn read(&mut self, address: u8, bytes: &mut [u8]) -> Result<()> {
        self.read_after_start(address, bytes)
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<()> {
        if write.is_empty() {
            return self.read(address, read);
        }
        if read.is_empty() {
            return self.write(address, write);
        }
        if let Err(error) = self.write_without_stop(address, write) {
            self.stop();
            return Err(error);
        }
        self.read_after_start(address, read)
    }
}

fn wait_for(mut condition: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..SYNC_TIMEOUT {
        if condition() {
            return Ok(());
        }
    }
    Err(Error::Platform)
}
