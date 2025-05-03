use core::ptr::{read_volatile, write_volatile};

use anyhal::hal::{
    gpio::{
        AlternateConfig, InputPin as InputPinContract, OutputPin as OutputPinContract, OutputType,
        Pin, Port, Pull, Speed,
    },
    Result,
};

const MODER: usize = 0x00;
const OTYPER: usize = 0x04;
const OSPEEDR: usize = 0x08;
const PUPDR: usize = 0x0c;
const IDR: usize = 0x10;
const BSRR: usize = 0x18;
const AFRL: usize = 0x20;
const AFRH: usize = 0x24;

fn port_base(port: Port) -> usize {
    use stm32h5::stm32h563 as pac;
    match port {
        Port::A => pac::GPIOA::ptr() as usize,
        Port::B => pac::GPIOB::ptr() as usize,
        Port::C => pac::GPIOC::ptr() as usize,
        Port::D => pac::GPIOD::ptr() as usize,
        Port::E => pac::GPIOE::ptr() as usize,
        Port::F => pac::GPIOF::ptr() as usize,
        Port::G => pac::GPIOG::ptr() as usize,
        Port::H => pac::GPIOH::ptr() as usize,
        Port::I => pac::GPIOI::ptr() as usize,
    }
}

unsafe fn enable_port(port: Port) {
    let rcc = unsafe { &*stm32h5::stm32h563::RCC::ptr() };
    rcc.ahb2enr().modify(|_, w| match port {
        Port::A => w.gpioaen().set_bit(),
        Port::B => w.gpioben().set_bit(),
        Port::C => w.gpiocen().set_bit(),
        Port::D => w.gpioden().set_bit(),
        Port::E => w.gpioeen().set_bit(),
        Port::F => w.gpiofen().set_bit(),
        Port::G => w.gpiogen().set_bit(),
        Port::H => w.gpiohen().set_bit(),
        Port::I => w.gpioien().set_bit(),
    });
    let _ = rcc.ahb2enr().read().bits();
}

unsafe fn write_field(address: usize, shift: u32, width: u32, value: u32) {
    let register = address as *mut u32;
    let mask = ((1_u32 << width) - 1) << shift;
    let current = unsafe { read_volatile(register) };
    unsafe { write_volatile(register, (current & !mask) | ((value << shift) & mask)) };
}

fn pull_bits(pull: Pull) -> u32 {
    match pull {
        Pull::None => 0,
        Pull::Up => 1,
        Pull::Down => 2,
    }
}

pub struct OutputPin {
    base: usize,
    mask: u32,
    high: bool,
}

impl OutputPin {
    /// # Safety
    /// Caller must exclusively own the pin.
    pub unsafe fn claim(pin: Pin, initially_high: bool) -> Self {
        unsafe { enable_port(pin.port()) };
        let base = port_base(pin.port());
        let mask = 1_u32 << pin.index();
        unsafe {
            write_volatile(
                (base + BSRR) as *mut u32,
                if initially_high { mask } else { mask << 16 },
            );
            write_field(base + MODER, u32::from(pin.index()) * 2, 2, 1);
        }
        Self {
            base,
            mask,
            high: initially_high,
        }
    }
}

impl OutputPinContract for OutputPin {
    fn set_low(&mut self) -> Result<()> {
        unsafe { write_volatile((self.base + BSRR) as *mut u32, self.mask << 16) };
        self.high = false;
        Ok(())
    }
    fn set_high(&mut self) -> Result<()> {
        unsafe { write_volatile((self.base + BSRR) as *mut u32, self.mask) };
        self.high = true;
        Ok(())
    }
    fn toggle(&mut self) -> Result<()> {
        if self.high {
            self.set_low()
        } else {
            self.set_high()
        }
    }
    fn is_set_high(&self) -> bool {
        self.high
    }
}

pub struct InputPin {
    base: usize,
    mask: u32,
}

impl InputPin {
    /// # Safety
    /// Caller must exclusively own the pin.
    pub unsafe fn claim(pin: Pin, pull: Pull) -> Self {
        unsafe { enable_port(pin.port()) };
        let base = port_base(pin.port());
        unsafe {
            write_field(base + MODER, u32::from(pin.index()) * 2, 2, 0);
            write_field(base + PUPDR, u32::from(pin.index()) * 2, 2, pull_bits(pull));
        }
        Self {
            base,
            mask: 1_u32 << pin.index(),
        }
    }
}

impl InputPinContract for InputPin {
    fn is_high(&self) -> bool {
        unsafe { read_volatile((self.base + IDR) as *const u32) & self.mask != 0 }
    }
}

pub struct AlternatePin {
    pin: Pin,
    config: AlternateConfig,
}

impl AlternatePin {
    /// # Safety
    /// Caller must exclusively own the pin and select a valid function for it.
    pub unsafe fn claim(pin: Pin, config: AlternateConfig) -> Result<Self> {
        unsafe { enable_port(pin.port()) };
        let base = port_base(pin.port());
        let index = u32::from(pin.index());
        let (afr, shift) = if index < 8 {
            (AFRL, index * 4)
        } else {
            (AFRH, (index - 8) * 4)
        };
        let speed = match config.speed {
            Speed::Low => 0,
            Speed::Medium => 1,
            Speed::High => 2,
            Speed::VeryHigh => 3,
        };
        unsafe {
            write_field(base + afr, shift, 4, u32::from(config.function.number()));
            write_field(
                base + OTYPER,
                index,
                1,
                u32::from(config.output_type == OutputType::OpenDrain),
            );
            write_field(base + OSPEEDR, index * 2, 2, speed);
            write_field(base + PUPDR, index * 2, 2, pull_bits(config.pull));
            write_field(base + MODER, index * 2, 2, 2);
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
