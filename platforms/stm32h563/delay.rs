use anyhal::hal::{delay::DelayMs, Error, Result};
use core::ptr::{read_volatile, write_volatile};

const CSR: *mut u32 = 0xe000_e010 as *mut u32;
const RVR: *mut u32 = 0xe000_e014 as *mut u32;
const CVR: *mut u32 = 0xe000_e018 as *mut u32;

pub struct SysTickDelay {
    ticks_per_ms: u32,
}

impl SysTickDelay {
    /// # Safety
    /// SysTick must be exclusively owned and `core_hz` must be accurate.
    pub unsafe fn claim(core_hz: u32) -> Result<Self> {
        let ticks = core_hz / 1_000;
        if ticks == 0 || ticks > 0x0100_0000 {
            return Err(Error::InvalidArgument);
        }
        unsafe { write_volatile(CSR, 0) };
        Ok(Self {
            ticks_per_ms: ticks,
        })
    }
}
impl DelayMs for SysTickDelay {
    fn delay_ms(&mut self, milliseconds: u32) -> Result<()> {
        for _ in 0..milliseconds {
            unsafe {
                write_volatile(RVR, self.ticks_per_ms - 1);
                write_volatile(CVR, 0);
                write_volatile(CSR, 0b101);
                while read_volatile(CSR) & (1 << 16) == 0 {}
                write_volatile(CSR, 0);
            }
        }
        Ok(())
    }
}
