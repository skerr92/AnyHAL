//! Blocking SERCOM4 SPI controller for PB12/PB13/PB14.

use crate::{AlternatePin, Clocks};
use anyhal::hal::{
    gpio::{AlternateConfig, AlternateFunction, OutputType, Pin, Port, Pull, Speed},
    spi::{BitOrder, Config, Phase, Polarity, SpiBus},
    Error, Result,
};

const SERCOM4_CORE_GCLK_ID: usize = 34;
const IO_TIMEOUT: u32 = 1_000_000;

pub struct Sercom4Spi {
    _mosi: AlternatePin,
    _sck: AlternatePin,
    _miso: AlternatePin,
}

impl Sercom4Spi {
    /// Claims SERCOM4 with MOSI=PB12/pad0, SCK=PB13/pad1, MISO=PB14/pad2.
    ///
    /// # Safety
    ///
    /// The caller must exclusively own SERCOM4, its clock channel, and all
    /// three pins. No chip-select pin is managed by this bus object.
    pub unsafe fn claim(
        mosi: Pin,
        sck: Pin,
        miso: Pin,
        clocks: Clocks,
        config: Config,
    ) -> Result<Self> {
        if mosi != Pin::new(Port::B, 12)
            || sck != Pin::new(Port::B, 13)
            || miso != Pin::new(Port::B, 14)
        {
            return Err(Error::InvalidArgument);
        }
        let divider = clocks
            .core_hz()
            .checked_div(
                config
                    .frequency_hz
                    .checked_mul(2)
                    .ok_or(Error::InvalidArgument)?,
            )
            .and_then(|value| value.checked_sub(1))
            .ok_or(Error::InvalidArgument)?;
        if divider > u8::MAX.into() {
            return Err(Error::InvalidArgument);
        }

        let pin_config = AlternateConfig {
            function: AlternateFunction::new(2).ok_or(Error::Platform)?,
            pull: Pull::None,
            output_type: OutputType::PushPull,
            speed: Speed::VeryHigh,
        };
        let mosi = unsafe { AlternatePin::claim(mosi, pin_config)? };
        let sck = unsafe { AlternatePin::claim(sck, pin_config)? };
        let miso = unsafe { AlternatePin::claim(miso, pin_config)? };

        let gclk = unsafe { &*atsamd51p19a::Gclk::ptr() };
        let mclk = unsafe { &*atsamd51p19a::Mclk::ptr() };
        let spi = unsafe { &*atsamd51p19a::Sercom4::ptr() }.spim();
        mclk.apbdmask().modify(|_, w| w.sercom4_().set_bit());
        gclk.pchctrl(SERCOM4_CORE_GCLK_ID)
            .write(|w| w.gen().gclk0().chen().set_bit());

        spi.ctrla().write(|w| w.swrst().set_bit());
        wait_for(|| !spi.syncbusy().read().swrst().bit())?;
        spi.ctrla().write(|w| {
            let w = w.mode().spi_master().dopo().pad0().dipo().pad2();
            let w = match config.polarity {
                Polarity::IdleLow => w.cpol().idle_low(),
                Polarity::IdleHigh => w.cpol().idle_high(),
            };
            let w = match config.phase {
                Phase::CaptureOnFirstTransition => w.cpha().leading_edge(),
                Phase::CaptureOnSecondTransition => w.cpha().trailing_edge(),
            };
            match config.bit_order {
                BitOrder::MostSignificantFirst => w.dord().msb(),
                BitOrder::LeastSignificantFirst => w.dord().lsb(),
            }
        });
        spi.ctrlb().write(|w| w.rxen().set_bit());
        wait_for(|| !spi.syncbusy().read().ctrlb().bit())?;
        spi.baud().write(|w| w.baud().bits(divider as u8));
        spi.ctrla().modify(|_, w| w.enable().set_bit());
        wait_for(|| !spi.syncbusy().read().enable().bit())?;

        Ok(Self {
            _mosi: mosi,
            _sck: sck,
            _miso: miso,
        })
    }

    fn registers(&self) -> &atsamd51p19a::sercom0::Spim {
        unsafe { &*atsamd51p19a::Sercom4::ptr() }.spim()
    }
}

impl SpiBus for Sercom4Spi {
    fn transfer_byte(&mut self, byte: u8) -> Result<u8> {
        let spi = self.registers();
        wait_for(|| spi.intflag().read().dre().bit())?;
        spi.data()
            .write(|w| unsafe { w.data().bits(u32::from(byte)) });
        wait_for(|| spi.intflag().read().rxc().bit())?;
        if spi.status().read().bufovf().bit() {
            spi.status().write(|w| w.bufovf().set_bit());
            return Err(Error::Bus);
        }
        Ok(spi.data().read().data().bits() as u8)
    }
}

fn wait_for(mut condition: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..IO_TIMEOUT {
        if condition() {
            return Ok(());
        }
    }
    Err(Error::Platform)
}
