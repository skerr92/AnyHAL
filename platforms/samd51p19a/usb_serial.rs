//! Native-USB CDC-ACM serial console for the SAMD51.

use atsamd_hal::{
    clock::{ClockGenId, ClockSource, GenericClockController},
    gpio::Pins,
    pac,
    usb::UsbBus,
};
use static_cell::StaticCell;
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
    UsbError,
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use crate::Clocks;
use anyhal::hal::{serial::SerialConsole, Error, Result};

static USB_ALLOCATOR: StaticCell<UsbBusAllocator<UsbBus>> = StaticCell::new();

pub struct UsbCdcConsole {
    serial: SerialPort<'static, UsbBus>,
    device: UsbDevice<'static, UsbBus>,
}

impl UsbCdcConsole {
    /// Claims native USB, PA24/PA25, and the complete clock tree.
    ///
    /// This configures the SAMD51 core at 120 MHz and USB from a dedicated
    /// 48 MHz DFLL generator. It must be called before any other peripheral is
    /// configured and at most once after reset.
    ///
    /// # Safety
    ///
    /// The caller must own the clock, PORT, MCLK, NVMCTRL, oscillator, and USB
    /// peripherals as well as PA24 and PA25.
    pub unsafe fn claim() -> Result<(Self, Clocks)> {
        let mut peripherals = unsafe { pac::Peripherals::steal() };
        let pins = Pins::new(peripherals.port);
        let mut clocks = GenericClockController::with_internal_32kosc(
            peripherals.gclk,
            &mut peripherals.mclk,
            &mut peripherals.osc32kctrl,
            &mut peripherals.oscctrl,
            &mut peripherals.nvmctrl,
        );
        let usb_generator = clocks
            .configure_gclk_divider_and_source(ClockGenId::Gclk2, 1, ClockSource::Dfll, true)
            .ok_or(Error::Platform)?;
        let usb_clock = clocks.usb(&usb_generator).ok_or(Error::Platform)?;
        let bus = USB_ALLOCATOR.init(UsbBusAllocator::new(UsbBus::new(
            &usb_clock,
            &mut peripherals.mclk,
            pins.pa24,
            pins.pa25,
            peripherals.usb,
        )));
        let serial = SerialPort::new(bus);
        let strings = [StringDescriptors::default()
            .manufacturer("AnyHAL")
            .product("SAMD51 Test Console")
            .serial_number("ANYHAL-SAMD51")];
        let device = UsbDeviceBuilder::new(bus, UsbVidPid(0x16c0, 0x27dd))
            .strings(&strings)
            .map_err(|_| Error::Platform)?
            .device_class(USB_CLASS_CDC)
            .build();

        Ok((Self { serial, device }, Clocks::new(120_000_000)))
    }

    pub fn is_terminal_open(&self) -> bool {
        self.serial.dtr()
    }
}

impl SerialConsole for UsbCdcConsole {
    fn poll(&mut self) -> bool {
        self.device.poll(&mut [&mut self.serial])
    }

    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        self.serial.write(bytes).map_err(map_usb_error)
    }

    fn read(&mut self, bytes: &mut [u8]) -> Result<usize> {
        self.serial.read(bytes).map_err(map_usb_error)
    }
}

fn map_usb_error(error: UsbError) -> Error {
    match error {
        UsbError::WouldBlock => Error::WouldBlock,
        _ => Error::Platform,
    }
}
