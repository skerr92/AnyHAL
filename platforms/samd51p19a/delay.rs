use core::ptr::{read_volatile, write_volatile};

use anyhal::hal::{delay::DelayMs, Error, Result};

const SYST_CSR: *mut u32 = 0xe000_e010 as *mut u32;
const SYST_RVR: *mut u32 = 0xe000_e014 as *mut u32;
const SYST_CVR: *mut u32 = 0xe000_e018 as *mut u32;
const SYST_ENABLE_CLKSOURCE: u32 = 0b101;
const SYST_COUNTFLAG: u32 = 1 << 16;
const SYST_MAX_RELOAD: u32 = 0x00ff_ffff;

pub struct SysTickDelay {
    ticks_per_ms: u32,
}

impl SysTickDelay {
    /// Claims the Cortex-M SysTick peripheral for blocking delays.
    ///
    /// # Safety
    ///
    /// SysTick must have no other owner and `core_hz` must describe its clock.
    pub unsafe fn claim(core_hz: u32) -> Result<Self> {
        let ticks_per_ms = core_hz / 1_000;
        if ticks_per_ms == 0 || ticks_per_ms > SYST_MAX_RELOAD + 1 {
            return Err(Error::InvalidArgument);
        }
        unsafe { write_volatile(SYST_CSR, 0) };
        Ok(Self { ticks_per_ms })
    }
}

impl DelayMs for SysTickDelay {
    fn delay_ms(&mut self, milliseconds: u32) -> Result<()> {
        for _ in 0..milliseconds {
            unsafe {
                write_volatile(SYST_RVR, self.ticks_per_ms - 1);
                write_volatile(SYST_CVR, 0);
                write_volatile(SYST_CSR, SYST_ENABLE_CLKSOURCE);
                while read_volatile(SYST_CSR) & SYST_COUNTFLAG == 0 {}
                write_volatile(SYST_CSR, 0);
            }
        }
        Ok(())
    }
}
