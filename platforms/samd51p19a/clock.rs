use anyhal::hal::{Error, Result};

const DFLL48M_HZ: u32 = 48_000_000;
const SYNC_TIMEOUT: u32 = 5_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Clocks {
    core_hz: u32,
}

impl Clocks {
    pub(crate) const fn new(core_hz: u32) -> Self {
        Self { core_hz }
    }

    pub const fn core_hz(self) -> u32 {
        self.core_hz
    }
}

/// Reconfigures GCLK0 to the DFLL48M in open-loop mode.
///
/// # Safety
///
/// The caller must exclusively own the system clocks, and active peripherals
/// must not depend on the clock configuration being replaced.
pub unsafe fn configure_dfll48m() -> Result<Clocks> {
    let gclk = unsafe { &*atsamd51p19a::Gclk::ptr() };
    let oscctrl = unsafe { &*atsamd51p19a::Oscctrl::ptr() };
    let mclk = unsafe { &*atsamd51p19a::Mclk::ptr() };

    gclk.genctrl(0)
        .write(|w| w.src().osculp32k().genen().set_bit());
    wait_for(|| gclk.syncbusy().read().genctrl().bits() & 1 == 0)?;

    oscctrl.dfllctrla().write(|w| w.enable().clear_bit());
    wait_for(|| !oscctrl.dfllsync().read().enable().bit())?;
    oscctrl.dfllctrlb().reset();
    wait_for(|| !oscctrl.dfllsync().read().dfllctrlb().bit())?;
    oscctrl.dfllmul().reset();
    wait_for(|| !oscctrl.dfllsync().read().dfllmul().bit())?;
    oscctrl
        .dfllctrla()
        .write(|w| w.ondemand().clear_bit().enable().set_bit());
    wait_for(|| !oscctrl.dfllsync().read().enable().bit())?;

    let calibration = oscctrl.dfllval().read().bits();
    oscctrl.dfllval().write(|w| unsafe { w.bits(calibration) });
    wait_for(|| !oscctrl.dfllsync().read().dfllval().bit())?;
    wait_for(|| oscctrl.status().read().dfllrdy().bit())?;

    gclk.genctrl(0).write(|w| {
        w.src()
            .dfll()
            .idc()
            .set_bit()
            .divsel()
            .div1()
            .genen()
            .set_bit()
    });
    wait_for(|| gclk.syncbusy().read().genctrl().bits() & 1 == 0)?;
    mclk.cpudiv().write(|w| w.div().div1());

    Ok(Clocks::new(DFLL48M_HZ))
}

fn wait_for(mut condition: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..SYNC_TIMEOUT {
        if condition() {
            return Ok(());
        }
    }
    Err(Error::Platform)
}
