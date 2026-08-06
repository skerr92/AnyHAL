#![no_std]
#![no_main]

use anyhal::hal::{adc::OneShotAdc, delay::DelayMs, gpio::OutputPin, serial::SerialConsole, Error};
use core::{arch::asm, fmt::Write, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

const REPORT: &[u8] = b"\r\nAnyHAL ADC test\r\n\
[PASS] ADC0 configured for 12-bit one-shot conversion\r\n\
[PASS] input is board A0 / PB08 / ADC0-AIN2\r\n\
[RUN] reporting a 16-sample average every 250 ms\r\n\
Drive A0 only between GND and 3.3 V.\r\n";
const ADC_FAILURE: &[u8] = b"\r\n[FAIL] ADC0 initialization timed out\r\n";

fn firmware_main() -> ! {
    let (mut console, clocks) =
        unsafe { anyhal_samd51p19a::UsbCdcConsole::claim().unwrap_or_else(|_| panic!()) };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };

    // Give the USB device stack time to enumerate before touching ADC0. This
    // also ensures ADC setup failures remain observable from the host.
    for _ in 0..500 {
        console.poll();
        delay.delay_ms(1).unwrap_or_else(|_| panic!());
    }

    let mut adc =
        match unsafe { anyhal_samd51p19a::Adc0Input::claim(anyhal_board_feather_samd51p19::A0) } {
            Ok(adc) => adc,
            Err(_) => adc_failure_loop(&mut console, &mut led, &mut delay),
        };
    let mut terminal_was_open = false;

    loop {
        console.poll();
        let terminal_open = console.is_terminal_open();
        if terminal_open && !terminal_was_open {
            write_all(&mut console, REPORT);
        }
        terminal_was_open = terminal_open;

        let mut sum = 0_u32;
        for _ in 0..16 {
            sum += u32::from(adc.read().unwrap_or_else(|_| panic!()));
        }
        let raw = (sum / 16) as u16;
        let millivolts = u32::from(raw) * adc.reference_mv() / 4095;
        if raw >= 2048 {
            led.set_high().unwrap_or_else(|_| panic!());
        } else {
            led.set_low().unwrap_or_else(|_| panic!());
        }

        if terminal_open {
            let mut line = TextBuffer::new();
            writeln!(&mut line, "A0 raw={raw:4}  voltage={millivolts:4} mV\r")
                .unwrap_or_else(|_| panic!());
            write_all(&mut console, line.as_bytes());
        }
        delay.delay_ms(250).unwrap_or_else(|_| panic!());
    }
}

fn adc_failure_loop(
    console: &mut anyhal_samd51p19a::UsbCdcConsole,
    led: &mut anyhal_samd51p19a::OutputPin,
    delay: &mut anyhal_samd51p19a::SysTickDelay,
) -> ! {
    let mut reported = false;
    loop {
        console.poll();
        if console.is_terminal_open() && !reported {
            write_all(console, ADC_FAILURE);
            reported = true;
        } else if !console.is_terminal_open() {
            reported = false;
        }
        led.toggle().unwrap_or_else(|_| panic!());
        for _ in 0..100 {
            console.poll();
            delay.delay_ms(1).unwrap_or_else(|_| panic!());
        }
    }
}

fn write_all(console: &mut anyhal_samd51p19a::UsbCdcConsole, bytes: &[u8]) {
    let mut offset = 0;
    while offset < bytes.len() {
        console.poll();
        match console.write(&bytes[offset..]) {
            Ok(count) => offset += count,
            Err(Error::WouldBlock) => {}
            Err(_) => panic!(),
        }
    }
}

struct TextBuffer {
    bytes: [u8; 64],
    len: usize,
}

impl TextBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl Write for TextBuffer {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let end = self.len + text.len();
        if end > self.bytes.len() {
            return Err(core::fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
