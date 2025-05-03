use core::ptr::{read_volatile, write_volatile};

use anyhal::hal::{
    gpio::{
        AlternateConfig, InputPin as InputPinContract, OutputPin as OutputPinContract, OutputType,
        Pin, Port, Pull, Speed,
    },
    Error, Result,
};

const PORT_BASE: usize = 0x4100_8000;
const PORT_GROUP_SIZE: usize = 0x80;
const DIRCLR: usize = 0x04;
const DIRSET: usize = 0x08;
const OUTCLR: usize = 0x14;
const OUTSET: usize = 0x18;
const OUTTGL: usize = 0x1c;
const IN: usize = 0x20;
const PINCFG: usize = 0x40;
const PMUX: usize = 0x30;
const PINCFG_PMUXEN: u8 = 1 << 0;
const PINCFG_INEN: u8 = 1 << 1;
const PINCFG_PULLEN: u8 = 1 << 2;
const PINCFG_DRVSTR: u8 = 1 << 6;

fn group_for(pin: Pin) -> Result<usize> {
    match pin.port() {
        Port::A | Port::B | Port::C | Port::D => {
            Ok(PORT_BASE + pin.port() as usize * PORT_GROUP_SIZE)
        }
        _ => Err(Error::InvalidArgument),
    }
}

pub struct AlternatePin {
    pin: Pin,
    config: AlternateConfig,
}

impl AlternatePin {
    /// Claims a pin for a peripheral function (0=A, 1=B, and so on).
    ///
    /// # Safety
    ///
    /// Caller must exclusively own the pin and choose a function supported by
    /// that pin. SAMD51 hardware does not provide native open-drain GPIO mode.
    pub unsafe fn claim(pin: Pin, config: AlternateConfig) -> Result<Self> {
        if config.output_type == OutputType::OpenDrain || config.function.number() > 13 {
            return Err(Error::Unsupported);
        }
        let group = group_for(pin)?;
        let index = usize::from(pin.index());
        let pmux = (group + PMUX + index / 2) as *mut u8;
        let shift = if index & 1 == 0 { 0 } else { 4 };
        unsafe {
            let old = read_volatile(pmux);
            let mask = 0x0f << shift;
            write_volatile(pmux, (old & !mask) | (config.function.number() << shift));
            match config.pull {
                Pull::Up => write_volatile((group + OUTSET) as *mut u32, 1 << pin.index()),
                Pull::Down => write_volatile((group + OUTCLR) as *mut u32, 1 << pin.index()),
                Pull::None => {}
            }
            let pin_config = PINCFG_PMUXEN
                | PINCFG_INEN
                | if config.pull == Pull::None {
                    0
                } else {
                    PINCFG_PULLEN
                }
                | if config.speed == Speed::VeryHigh {
                    PINCFG_DRVSTR
                } else {
                    0
                };
            write_volatile((group + PINCFG + index) as *mut u8, pin_config);
        }
        Ok(Self { pin, config })
    }

    pub const fn pin(&self) -> Pin {
        self.pin
    }
    pub const fn config(&self) -> AlternateConfig {
        self.config
    }
}

pub struct InputPin {
    group: usize,
    mask: u32,
}

impl InputPin {
    /// Claims a physical pin as a digital input with optional internal pull.
    ///
    /// # Safety
    ///
    /// The caller must ensure no other owner accesses or reconfigures this pin.
    pub unsafe fn claim(pin: Pin, pull: Pull) -> Self {
        let group = group_for(pin).expect("pin port is unavailable on SAMD51");
        let mask = 1_u32 << pin.index();
        unsafe {
            write_volatile((group + DIRCLR) as *mut u32, mask);
            match pull {
                Pull::Up => write_volatile((group + OUTSET) as *mut u32, mask),
                Pull::Down => write_volatile((group + OUTCLR) as *mut u32, mask),
                Pull::None => {}
            }
            let config = PINCFG_INEN | if pull == Pull::None { 0 } else { PINCFG_PULLEN };
            write_volatile(
                (group + PINCFG + usize::from(pin.index())) as *mut u8,
                config,
            );
        }
        Self { group, mask }
    }
}

impl InputPinContract for InputPin {
    fn is_high(&self) -> bool {
        unsafe { read_volatile((self.group + IN) as *const u32) & self.mask != 0 }
    }
}

pub struct OutputPin {
    group: usize,
    mask: u32,
    high: bool,
}

impl OutputPin {
    /// Claims a physical pin as a push-pull output.
    ///
    /// # Safety
    ///
    /// The caller must ensure no other owner accesses or reconfigures this pin.
    pub unsafe fn claim(pin: Pin, initially_high: bool) -> Self {
        let group = group_for(pin).expect("pin port is unavailable on SAMD51");
        let mask = 1_u32 << pin.index();
        let initial_register = if initially_high { OUTSET } else { OUTCLR };
        unsafe {
            write_volatile((group + initial_register) as *mut u32, mask);
            write_volatile((group + DIRSET) as *mut u32, mask);
        }
        Self {
            group,
            mask,
            high: initially_high,
        }
    }
}

impl OutputPinContract for OutputPin {
    fn set_low(&mut self) -> Result<()> {
        unsafe { write_volatile((self.group + OUTCLR) as *mut u32, self.mask) };
        self.high = false;
        Ok(())
    }

    fn set_high(&mut self) -> Result<()> {
        unsafe { write_volatile((self.group + OUTSET) as *mut u32, self.mask) };
        self.high = true;
        Ok(())
    }

    fn toggle(&mut self) -> Result<()> {
        unsafe { write_volatile((self.group + OUTTGL) as *mut u32, self.mask) };
        self.high = !self.high;
        Ok(())
    }

    fn is_set_high(&self) -> bool {
        self.high
    }
}
