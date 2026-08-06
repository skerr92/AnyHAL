#![no_std]
#![no_main]

use anyhal::hal::{
    dac::AnalogOutput, delay::DelayMs, gpio::OutputPin, serial::SerialConsole, Error,
};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

const REPORT: &[u8] = b"\r\nAnyHAL DAC test\r\n\
[PASS] DAC channel 1 configured for 12-bit output\r\n\
[PASS] output is board A2 / PA05 / VOUT1\r\n\
[RUN] generating a stepped 0-3.3 V triangle wave\r\n\
Probe A2 with an oscilloscope; do not externally drive the pin.\r\n";
const DAC_FAILURE: &[u8] = b"\r\n[FAIL] DAC channel 1 initialization timed out\r\n";

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

    let mut dac =
        match unsafe { anyhal_samd51p19a::Dac1Output::claim(anyhal_board_feather_samd51p19::A2) } {
            Ok(dac) => dac,
            Err(_) => failure_loop(&mut console, &mut led, &mut delay),
        };
    let mut terminal_was_open = false;
    let mut value = 0_u16;
    let mut rising = true;

    loop {
        console.poll();
        let terminal_open = console.is_terminal_open();
        if terminal_open && !terminal_was_open {
            write_all(&mut console, REPORT);
        }
        terminal_was_open = terminal_open;

        dac.set_value(value).unwrap_or_else(|_| panic!());
        if rising {
            let next = value.saturating_add(32).min(4095);
            if next == 4095 {
                rising = false;
                led.toggle().unwrap_or_else(|_| panic!());
            }
            value = next;
        } else {
            let next = value.saturating_sub(32);
            if next == 0 {
                rising = true;
                led.toggle().unwrap_or_else(|_| panic!());
            }
            value = next;
        }
        delay.delay_ms(5).unwrap_or_else(|_| panic!());
    }
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
            write_all(console, DAC_FAILURE);
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

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
