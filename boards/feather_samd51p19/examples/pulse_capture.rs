#![no_std]
#![no_main]

use anyhal::hal::{
    capture::{PulseCapture, PulseMeasurement},
    delay::DelayMs,
    gpio::OutputPin,
    pwm::PwmOutput,
    serial::SerialConsole,
    Error,
};
use core::{arch::asm, fmt::Write, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

const REPORT: &[u8] = b"\r\nAnyHAL pulse capture test\r\n\
[RUN] A1/PB09 generates a 50 Hz servo pulse with TC4\r\n\
[RUN] A3/PA06 captures period and high time with EIC/EVSYS/TC5\r\n\
Connect a jumper from A1 to A3. Pulse width alternates between 1000 and 2000 us.\r\n";
const CAPTURE_FAILURE: &[u8] = b"\r\n[FAIL] pulse capture initialization timed out\r\n";

fn firmware_main() -> ! {
    let (mut console, clocks) =
        unsafe { anyhal_samd51p19a::UsbCdcConsole::claim().unwrap_or_else(|_| panic!()) };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };

    for _ in 0..500 {
        console.poll();
        delay.delay_ms(1).unwrap_or_else(|_| panic!());
    }

    let mut pwm = unsafe {
        anyhal_samd51p19a::Tc4Pwm::claim(anyhal_board_feather_samd51p19::A1, clocks, 50)
            .unwrap_or_else(|_| panic!())
    };
    set_pulse_us(&mut pwm, 1_000);
    pwm.enable().unwrap_or_else(|_| panic!());

    let mut capture = match unsafe {
        anyhal_samd51p19a::Tc5PulseCapture::claim(anyhal_board_feather_samd51p19::A3, clocks)
    } {
        Ok(capture) => capture,
        Err(_) => failure_loop(&mut console, &mut led, &mut delay),
    };

    let mut latest: Option<PulseMeasurement> = None;
    let mut elapsed_ms = 0_u32;
    let mut pulse_us = 1_000_u32;
    let mut terminal_was_open = false;

    loop {
        console.poll();
        let terminal_open = console.is_terminal_open();
        if terminal_open && !terminal_was_open {
            write_all(&mut console, REPORT);
        }
        terminal_was_open = terminal_open;

        if let Some(sample) = capture.try_measure().unwrap_or_else(|_| panic!()) {
            latest = Some(sample);
        }

        if elapsed_ms != 0 && elapsed_ms.is_multiple_of(2_000) {
            pulse_us = if pulse_us == 1_000 { 2_000 } else { 1_000 };
            set_pulse_us(&mut pwm, pulse_us);
            led.toggle().unwrap_or_else(|_| panic!());
        }

        if terminal_open && elapsed_ms.is_multiple_of(500) {
            let mut line = TextBuffer::new();
            match latest {
                Some(sample) => writeln!(
                    &mut line,
                    "command={pulse_us:4} us  period={:5} us ({:5} ticks)  high={:4} us ({:4} ticks)\r",
                    sample.period_us(capture.tick_hz()),
                    sample.period_ticks,
                    sample.high_us(capture.tick_hz()),
                    sample.high_ticks
                ),
                None => writeln!(&mut line, "command={pulse_us:4} us  waiting for A1 -> A3 jumper\r"),
            }
            .unwrap_or_else(|_| panic!());
            write_all(&mut console, line.as_bytes());
        }

        delay.delay_ms(1).unwrap_or_else(|_| panic!());
        elapsed_ms = elapsed_ms.wrapping_add(1);
    }
}

fn set_pulse_us(pwm: &mut anyhal_samd51p19a::Tc4Pwm, pulse_us: u32) {
    let duty = (pulse_us * u32::from(u16::MAX) / 20_000) as u16;
    pwm.set_duty_cycle(duty).unwrap_or_else(|_| panic!());
}

fn failure_loop(
    console: &mut anyhal_samd51p19a::UsbCdcConsole,
    led: &mut anyhal_samd51p19a::OutputPin,
    delay: &mut anyhal_samd51p19a::SysTickDelay,
) -> ! {
    let mut reported = false;
    loop {
        console.poll();
        if console.is_terminal_open() && !reported {
            write_all(console, CAPTURE_FAILURE);
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
    bytes: [u8; 128],
    len: usize,
}

impl TextBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 128],
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
