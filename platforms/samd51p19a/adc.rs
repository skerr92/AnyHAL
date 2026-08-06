//! ADC0 one-shot conversion.

use core::ptr::{read_unaligned, read_volatile, write_volatile};

use crate::routes;
use anyhal::hal::{adc::OneShotAdc, gpio::Pin, gpio::Port, Error, Result};

const ADC0_GCLK_ID: usize = 40;
const GCLK2: u8 = 2;
const PORT_BASES: [usize; 4] = [0x4100_8000, 0x4100_8080, 0x4100_8100, 0x4100_8180];
const PMUX: usize = 0x30;
const PINCFG: usize = 0x40;
const PINCFG_PMUXEN: u8 = 1;
const NVM_CALIBRATION: usize = 0x0080_0080;
const SYNC_TIMEOUT: u32 = 1_000_000;

pub struct Adc0Input {
    channel: u8,
}

impl Adc0Input {
    /// Claims ADC0 and PB08/AIN2 using VDDANA as the 12-bit reference.
    ///
    /// # Safety
    ///
    /// The caller must exclusively own ADC0, PB08, the ADC0 clock channel, and
    /// the corresponding PORT multiplexer state. GCLK2 must already provide
    /// the USB-safe 48 MHz clock created by `UsbCdcConsole::claim`.
    pub unsafe fn claim(pin: Pin) -> Result<Self> {
        let route = routes::adc0(pin).ok_or(Error::InvalidArgument)?;

        configure_analog(pin, route.function)?;
        let gclk = unsafe { &*atsamd51p19a::Gclk::ptr() };
        let mclk = unsafe { &*atsamd51p19a::Mclk::ptr() };
        let adc = unsafe { &*atsamd51p19a::Adc0::ptr() };

        gclk.pchctrl(ADC0_GCLK_ID)
            .write(|w| unsafe { w.gen().bits(GCLK2) }.chen().set_bit());
        wait_for(|| gclk.pchctrl(ADC0_GCLK_ID).read().chen().bit())?;
        mclk.apbdmask().modify(|_, w| w.adc0_().set_bit());

        adc.ctrla().write(|w| w.swrst().set_bit());
        wait_for(|| {
            adc.syncbusy().read().bits() == 0 && adc.ctrla().read().swrst().bit_is_clear()
        })?;
        let calibration = unsafe { read_unaligned(NVM_CALIBRATION as *const u32) };
        adc.calib().write(|w| unsafe {
            w.biascomp().bits(((calibration >> 2) & 0x7) as u8);
            w.biasrefbuf().bits(((calibration >> 5) & 0x7) as u8);
            w.biasr2r().bits(((calibration >> 8) & 0x7) as u8)
        });
        wait_for(|| adc.syncbusy().read().bits() == 0)?;
        adc.ctrla().modify(|_, w| w.prescaler().div32());
        wait_for(|| adc.syncbusy().read().bits() == 0)?;
        adc.ctrlb().modify(|_, w| w.ressel()._12bit());
        wait_for(|| adc.syncbusy().read().ctrlb().bit_is_clear())?;
        adc.sampctrl().write(|w| unsafe { w.samplen().bits(4) });
        wait_for(|| adc.syncbusy().read().sampctrl().bit_is_clear())?;
        adc.inputctrl().write(|w| {
            unsafe { w.muxpos().bits(route.channel) }
                .muxneg()
                .gnd()
                .diffmode()
                .clear_bit()
        });
        wait_for(|| adc.syncbusy().read().inputctrl().bit_is_clear())?;
        adc.avgctrl().reset();
        wait_for(|| adc.syncbusy().read().avgctrl().bit_is_clear())?;
        adc.refctrl().write(|w| w.refsel().intvcc1());
        wait_for(|| adc.syncbusy().read().refctrl().bit_is_clear())?;
        adc.ctrla().modify(|_, w| w.enable().set_bit());
        wait_for(|| adc.syncbusy().read().enable().bit_is_clear())?;

        Ok(Self {
            channel: route.channel,
        })
    }
}

impl OneShotAdc for Adc0Input {
    fn resolution_bits(&self) -> u8 {
        12
    }

    fn reference_mv(&self) -> u32 {
        3300
    }

    fn read(&mut self) -> Result<u16> {
        let adc = unsafe { &*atsamd51p19a::Adc0::ptr() };
        adc.intflag()
            .write(|w| w.resrdy().set_bit().overrun().set_bit());
        adc.inputctrl()
            .modify(|_, w| unsafe { w.muxpos().bits(self.channel) });
        wait_for(|| adc.syncbusy().read().inputctrl().bit_is_clear())?;
        adc.swtrig().write(|w| w.start().set_bit());
        wait_for(|| adc.syncbusy().read().swtrig().bit_is_clear())?;
        wait_for(|| adc.intflag().read().resrdy().bit())?;
        Ok(adc.result().read().result().bits())
    }
}

fn configure_analog(pin: Pin, function: u8) -> Result<()> {
    let port_index = match pin.port() {
        Port::A => 0,
        Port::B => 1,
        Port::C => 2,
        Port::D => 3,
        _ => return Err(Error::InvalidArgument),
    };
    let base = PORT_BASES[port_index];
    let number = usize::from(pin.index());
    let pmux = (base + PMUX + number / 2) as *mut u8;
    unsafe {
        let old = read_volatile(pmux);
        let mux = if number & 1 == 0 {
            (old & 0xf0) | function
        } else {
            (old & 0x0f) | (function << 4)
        };
        write_volatile(pmux, mux);
        write_volatile((base + PINCFG + number) as *mut u8, PINCFG_PMUXEN);
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
