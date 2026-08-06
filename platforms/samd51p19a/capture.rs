//! TC5 period and pulse-width capture through EIC and EVSYS.

use anyhal::hal::{
    capture::{PulseCapture, PulseMeasurement},
    gpio::{AlternateConfig, AlternateFunction, OutputType, Pin, Pull, Speed},
    Error, Result,
};

use crate::{routes, AlternatePin, Clocks};

const TC4_TC5_GCLK_ID: usize = 30;
const EIC_EXTINT0_GENERATOR: u8 = 18;
const TC5_EVENT_USER: usize = 49;
const EVENT_CHANNEL: u8 = 0;
const PRESCALER: u32 = 64;
const SYNC_TIMEOUT: u32 = 1_000_000;

pub struct Tc5PulseCapture {
    _pin: AlternatePin,
    tick_hz: u32,
    pending_period: Option<u32>,
    pending_high: Option<u32>,
}

impl Tc5PulseCapture {
    /// Claims TC5, EIC EXTINT6, EVSYS channel 0, and PA06.
    ///
    /// # Safety
    ///
    /// The caller must exclusively own those peripherals and routing resources.
    pub unsafe fn claim(pin: Pin, clocks: Clocks) -> Result<Self> {
        let route = routes::eic(pin).ok_or(Error::InvalidArgument)?;

        let pin = unsafe {
            AlternatePin::claim(
                pin,
                AlternateConfig {
                    function: AlternateFunction::new(route.function).ok_or(Error::Platform)?,
                    pull: Pull::None,
                    output_type: OutputType::PushPull,
                    speed: Speed::High,
                },
            )?
        };

        let gclk = unsafe { &*atsamd51p19a::Gclk::ptr() };
        let mclk = unsafe { &*atsamd51p19a::Mclk::ptr() };
        let eic = unsafe { &*atsamd51p19a::Eic::ptr() };
        let evsys = unsafe { &*atsamd51p19a::Evsys::ptr() };
        let tc = unsafe { &*atsamd51p19a::Tc5::ptr() }.count16();

        mclk.apbamask().modify(|_, w| w.eic_().set_bit());
        mclk.apbbmask().modify(|_, w| w.evsys_().set_bit());
        mclk.apbcmask().modify(|_, w| w.tc5_().set_bit());
        gclk.pchctrl(TC4_TC5_GCLK_ID)
            .write(|w| unsafe { w.gen().bits(0) }.chen().set_bit());
        wait_for(|| gclk.pchctrl(TC4_TC5_GCLK_ID).read().chen().bit())?;

        eic.ctrla().write(|w| w.swrst().set_bit());
        wait_for(|| !eic.syncbusy().read().swrst().bit())?;
        // PPW needs a level that contains both rising and falling transitions.
        // Edge sensing emits narrow EIC event pulses, whose width rounds to
        // zero at the TC clock. High-level sensing forwards the input level.
        let config_index = usize::from(route.extint / 8);
        let config_shift = u32::from(route.extint % 8) * 4;
        eic.config(config_index).modify(|r, w| unsafe {
            w.bits((r.bits() & !(0x0f << config_shift)) | (4 << config_shift))
        });
        let extint_mask = 1_u16 << route.extint;
        eic.evctrl()
            .write(|w| unsafe { w.extinteo().bits(extint_mask) });
        eic.asynch()
            .write(|w| unsafe { w.asynch().bits(extint_mask) });
        eic.ctrla().write(|w| w.enable().set_bit());
        wait_for(|| !eic.syncbusy().read().enable().bit())?;

        evsys.channels(EVENT_CHANNEL.into()).channel().write(|w| {
            unsafe { w.evgen().bits(EIC_EXTINT0_GENERATOR + route.extint) }
                .path()
                .asynchronous()
                .edgsel()
                .no_evt_output()
        });
        evsys
            .user(TC5_EVENT_USER)
            .write(|w| unsafe { w.channel().bits(EVENT_CHANNEL + 1) });

        tc.ctrla().write(|w| w.swrst().set_bit());
        wait_for(|| !tc.ctrla().read().swrst().bit() && !tc.syncbusy().read().swrst().bit())?;
        tc.ctrla().modify(|_, w| {
            w.prescaler()
                .div64()
                .capten0()
                .set_bit()
                .capten1()
                .set_bit()
        });
        tc.evctrl().write(|w| w.evact().ppw().tcei().set_bit());
        tc.intflag().write(|w| w.mc0().set_bit().mc1().set_bit());
        tc.ctrla().modify(|_, w| w.enable().set_bit());
        wait_for(|| !tc.syncbusy().read().enable().bit())?;

        Ok(Self {
            _pin: pin,
            tick_hz: clocks.core_hz() / PRESCALER,
            pending_period: None,
            pending_high: None,
        })
    }

    fn timer() -> &'static atsamd51p19a::tc0::Count16 {
        unsafe { &*atsamd51p19a::Tc5::ptr() }.count16()
    }
}

impl PulseCapture for Tc5PulseCapture {
    fn tick_hz(&self) -> u32 {
        self.tick_hz
    }

    fn try_measure(&mut self) -> Result<Option<PulseMeasurement>> {
        let tc = Self::timer();
        let flags = tc.intflag().read();

        if flags.err().bit() {
            tc.intflag().write(|w| w.err().set_bit());
            self.pending_period = None;
            self.pending_high = None;
        }
        if flags.mc0().bit() {
            self.pending_period = Some(u32::from(tc.cc(0).read().cc().bits()));
            tc.intflag().write(|w| w.mc0().set_bit());
        }
        if flags.mc1().bit() {
            self.pending_high = Some(u32::from(tc.cc(1).read().cc().bits()));
            tc.intflag().write(|w| w.mc1().set_bit());
        }

        match (self.pending_period.take(), self.pending_high.take()) {
            (Some(period_ticks), Some(high_ticks)) => Ok(Some(PulseMeasurement {
                period_ticks,
                high_ticks,
            })),
            (period, high) => {
                self.pending_period = period;
                self.pending_high = high;
                Ok(None)
            }
        }
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
