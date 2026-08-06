//! TC4 single-channel match-PWM output.

use anyhal::hal::{
    gpio::{AlternateConfig, AlternateFunction, OutputType, Pin, Pull, Speed},
    pwm::PwmOutput,
    Error, Result,
};

use crate::{routes, AlternatePin, Clocks};

const TC4_TC5_GCLK_ID: usize = 30;
const SYNC_TIMEOUT: u32 = 1_000_000;

pub struct Tc4Pwm {
    _pin: AlternatePin,
    period_counts: u16,
    frequency_hz: u32,
    duty_cycle: u16,
    enabled: bool,
}

impl Tc4Pwm {
    /// Claims TC4 and a pin capable of routing TC4/WO1.
    ///
    /// # Safety
    ///
    /// The caller must exclusively own TC4, PB09, its PORT configuration, and
    /// the TC4/TC5 generic-clock channel.
    pub unsafe fn claim(pin: Pin, clocks: Clocks, frequency_hz: u32) -> Result<Self> {
        let route = routes::tc4(pin).ok_or(Error::InvalidArgument)?;
        if route.waveform_output != 1 || frequency_hz == 0 {
            return Err(Error::InvalidArgument);
        }

        let (divider, period_counts) = timer_parameters(clocks.core_hz(), frequency_hz)?;
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
        let tc = unsafe { &*atsamd51p19a::Tc4::ptr() }.count16();

        gclk.pchctrl(TC4_TC5_GCLK_ID)
            .write(|w| unsafe { w.gen().bits(0) }.chen().set_bit());
        wait_for(|| gclk.pchctrl(TC4_TC5_GCLK_ID).read().chen().bit())?;
        mclk.apbcmask().modify(|_, w| w.tc4_().set_bit());

        tc.ctrla().write(|w| w.swrst().set_bit());
        wait_for(|| !tc.ctrla().read().swrst().bit() && !tc.syncbusy().read().swrst().bit())?;
        tc.ctrla().modify(|_, w| match divider {
            1 => w.prescaler().div1(),
            2 => w.prescaler().div2(),
            4 => w.prescaler().div4(),
            8 => w.prescaler().div8(),
            16 => w.prescaler().div16(),
            64 => w.prescaler().div64(),
            256 => w.prescaler().div256(),
            1024 => w.prescaler().div1024(),
            _ => w,
        });
        tc.wave().write(|w| w.wavegen().mpwm());
        tc.cc(0)
            .write(|w| unsafe { w.cc().bits(period_counts - 1) });
        wait_for(|| !tc.syncbusy().read().cc0().bit())?;
        tc.cc(1).write(|w| unsafe { w.cc().bits(0) });
        wait_for(|| !tc.syncbusy().read().cc1().bit())?;

        Ok(Self {
            _pin: pin,
            period_counts,
            frequency_hz: clocks.core_hz() / divider / u32::from(period_counts),
            duty_cycle: 0,
            enabled: false,
        })
    }

    fn timer() -> &'static atsamd51p19a::tc0::Count16 {
        unsafe { &*atsamd51p19a::Tc4::ptr() }.count16()
    }
}

impl PwmOutput for Tc4Pwm {
    fn frequency_hz(&self) -> u32 {
        self.frequency_hz
    }

    fn duty_cycle(&self) -> u16 {
        self.duty_cycle
    }

    fn set_duty_cycle(&mut self, duty_cycle: u16) -> Result<()> {
        let native =
            (u32::from(duty_cycle) * u32::from(self.period_counts) / u32::from(u16::MAX)) as u16;
        let tc = Self::timer();
        tc.ccbuf(1).write(|w| unsafe { w.ccbuf().bits(native) });
        self.duty_cycle = duty_cycle;
        Ok(())
    }

    fn enable(&mut self) -> Result<()> {
        let tc = Self::timer();
        tc.ctrla().modify(|_, w| w.enable().set_bit());
        wait_for(|| !tc.syncbusy().read().enable().bit())?;
        self.enabled = true;
        Ok(())
    }

    fn disable(&mut self) -> Result<()> {
        let tc = Self::timer();
        tc.ctrla().modify(|_, w| w.enable().clear_bit());
        wait_for(|| !tc.syncbusy().read().enable().bit())?;
        self.enabled = false;
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

fn timer_parameters(clock_hz: u32, frequency_hz: u32) -> Result<(u32, u16)> {
    for divider in [1, 2, 4, 8, 16, 64, 256, 1024] {
        let counts = clock_hz / divider / frequency_hz;
        if (2..=u32::from(u16::MAX)).contains(&counts) {
            return Ok((divider, counts as u16));
        }
    }
    Err(Error::InvalidArgument)
}

fn wait_for(mut condition: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..SYNC_TIMEOUT {
        if condition() {
            return Ok(());
        }
    }
    Err(Error::Platform)
}
