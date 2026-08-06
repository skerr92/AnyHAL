//! DAC channel 1 voltage output.

use core::ptr::{read_volatile, write_volatile};

use crate::routes;
use anyhal::hal::{dac::AnalogOutput, gpio::Pin, gpio::Port, Error, Result};

const DAC_GCLK_ID: usize = 42;
const GCLK2: u8 = 2;
const PORTA_BASE: usize = 0x4100_8000;
const PMUX: usize = 0x30;
const PINCFG: usize = 0x40;
const PINCFG_PMUXEN: u8 = 1;
const CHANNEL: usize = 1;
const MAX_VALUE: u16 = 4095;
const SYNC_TIMEOUT: u32 = 1_000_000;

pub struct Dac1Output {
    value: u16,
}

impl Dac1Output {
    /// Claims DAC channel 1 and PA05/VOUT1 using VDDANA as reference.
    ///
    /// # Safety
    ///
    /// The caller must exclusively own DAC channel 1, PA05, the DAC clock
    /// channel, and the corresponding PORT multiplexer state. GCLK2 must
    /// already provide the 48 MHz clock created by `UsbCdcConsole::claim`.
    pub unsafe fn claim(pin: Pin) -> Result<Self> {
        let route = routes::dac1(pin).ok_or(Error::InvalidArgument)?;
        if usize::from(route.channel) != CHANNEL {
            return Err(Error::InvalidArgument);
        }

        configure_analog(pin, route.function)?;
        let gclk = unsafe { &*atsamd51p19a::Gclk::ptr() };
        let mclk = unsafe { &*atsamd51p19a::Mclk::ptr() };
        let dac = unsafe { &*atsamd51p19a::Dac::ptr() };

        gclk.pchctrl(DAC_GCLK_ID)
            .write(|w| unsafe { w.gen().bits(GCLK2) }.chen().set_bit());
        wait_for(|| gclk.pchctrl(DAC_GCLK_ID).read().chen().bit())?;
        mclk.apbdmask().modify(|_, w| w.dac_().set_bit());

        dac.ctrla().write(|w| w.swrst().set_bit());
        wait_for(|| {
            dac.syncbusy().read().swrst().bit_is_clear()
                && dac.ctrla().read().swrst().bit_is_clear()
        })?;
        dac.ctrlb().write(|w| w.refsel().vddana());
        dac.dacctrl(CHANNEL).write(|w| {
            w.leftadj()
                .clear_bit()
                .cctrl()
                .cc100k()
                .refresh()
                .refresh_2()
                .enable()
                .set_bit()
        });
        dac.ctrla().write(|w| w.enable().set_bit());
        wait_for(|| dac.syncbusy().read().enable().bit_is_clear())?;
        wait_for(|| dac.status().read().ready1().bit())?;
        dac.data(CHANNEL).write(|w| unsafe { w.data().bits(0) });
        wait_for(|| dac.syncbusy().read().data1().bit_is_clear())?;

        Ok(Self { value: 0 })
    }
}

impl AnalogOutput for Dac1Output {
    fn resolution_bits(&self) -> u8 {
        12
    }

    fn reference_mv(&self) -> u32 {
        3300
    }

    fn value(&self) -> u16 {
        self.value
    }

    fn set_value(&mut self, value: u16) -> Result<()> {
        if value > MAX_VALUE {
            return Err(Error::InvalidArgument);
        }
        let dac = unsafe { &*atsamd51p19a::Dac::ptr() };
        dac.data(CHANNEL).write(|w| unsafe { w.data().bits(value) });
        wait_for(|| dac.syncbusy().read().data1().bit_is_clear())?;
        wait_for(|| dac.status().read().eoc1().bit())?;
        self.value = value;
        Ok(())
    }
}

fn configure_analog(pin: Pin, function: u8) -> Result<()> {
    if pin.port() != Port::A {
        return Err(Error::InvalidArgument);
    }
    let number = usize::from(pin.index());
    let pmux = (PORTA_BASE + PMUX + number / 2) as *mut u8;
    unsafe {
        let old = read_volatile(pmux);
        let mux = if number & 1 == 0 {
            (old & 0xf0) | function
        } else {
            (old & 0x0f) | (function << 4)
        };
        write_volatile(pmux, mux);
        write_volatile((PORTA_BASE + PINCFG + number) as *mut u8, PINCFG_PMUXEN);
    }
    Ok(())
}

fn wait_for(mut condition: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..SYNC_TIMEOUT {
        if condition() {
            return Ok(());
        }
    }
    Err(Error::Platform)
}
