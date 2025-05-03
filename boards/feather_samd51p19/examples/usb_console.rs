#![no_std]
#![no_main]

use anyhal::hal::{delay::DelayMs, gpio::OutputPin, serial::SerialConsole, Error};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

const REPORT: &[u8] = b"\r\nAnyHAL SAMD51 USB test console\r\n\
[PASS] USB CDC-ACM enumerated\r\n\
[PASS] core clock: 120 MHz\r\n\
[PASS] LED: PB00\r\n\
[PASS] SPI: MOSI PB12, SCK PB13, MISO PB14\r\n\
Type into this terminal for an echo test.\r\n";

fn firmware_main() -> ! {
    let (mut console, clocks) =
        unsafe { anyhal_samd51p19a::UsbCdcConsole::claim().unwrap_or_else(|_| panic!()) };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };
    let mut report_offset = 0_usize;
    let mut terminal_was_open = false;
    let mut elapsed_ms = 0_u16;
    let mut input = [0_u8; 64];

    loop {
        console.poll();
        let terminal_open = console.is_terminal_open();
        if terminal_open && !terminal_was_open {
            report_offset = 0;
        }
        terminal_was_open = terminal_open;

        if terminal_open && report_offset < REPORT.len() {
            match console.write(&REPORT[report_offset..]) {
                Ok(count) => report_offset += count,
                Err(Error::WouldBlock) => {}
                Err(_) => panic!(),
            }
        } else if terminal_open {
            match console.read(&mut input) {
                Ok(count) => {
                    let mut offset = 0;
                    while offset < count {
                        console.poll();
                        match console.write(&input[offset..count]) {
                            Ok(written) => offset += written,
                            Err(Error::WouldBlock) => {}
                            Err(_) => panic!(),
                        }
                    }
                }
                Err(Error::WouldBlock) => {}
                Err(_) => panic!(),
            }
        }

        delay.delay_ms(1).unwrap_or_else(|_| panic!());
        elapsed_ms += 1;
        if elapsed_ms == 500 {
            led.toggle().unwrap_or_else(|_| panic!());
        } else if elapsed_ms >= 1_000 {
            led.toggle().unwrap_or_else(|_| panic!());
            elapsed_ms = 0;
            if terminal_open && report_offset == REPORT.len() {
                let _ = console.write(b"[PASS] USB polling heartbeat\r\n");
            }
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
