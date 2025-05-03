#![no_std]
#![no_main]

use anyhal::hal::{
    delay::DelayMs,
    gpio::OutputPin,
    serial::SerialConsole,
    spi::{Config, SpiBus},
    Error,
};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

const PASS_REPORT: &[u8] = b"\r\nAnyHAL SAMD51 SPI loopback\r\n\
[PASS] USB CDC console\r\n\
[PASS] SERCOM4 configured at 1 MHz, mode 0, MSB first\r\n\
[PASS] MOSI PB12 -> MISO PB14 loopback matched\r\n";
const FAIL_REPORT: &[u8] = b"\r\nAnyHAL SAMD51 SPI loopback\r\n\
[PASS] USB CDC console\r\n\
[PASS] SERCOM4 configured at 1 MHz, mode 0, MSB first\r\n\
[FAIL] loopback mismatch; jumper MOSI/PB12 to MISO/PB14 and reset\r\n";

fn firmware_main() -> ! {
    let (mut console, clocks) =
        unsafe { anyhal_samd51p19a::UsbCdcConsole::claim().unwrap_or_else(|_| panic!()) };
    let mut spi = unsafe {
        anyhal_samd51p19a::Sercom4Spi::claim(
            anyhal_board_feather_samd51p19::MOSI,
            anyhal_board_feather_samd51p19::SCK,
            anyhal_board_feather_samd51p19::MISO,
            clocks,
            Config::mode0(1_000_000),
        )
        .unwrap_or_else(|_| panic!())
    };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };

    let expected = [0x00, 0xff, 0x3c, 0xa5, 0x5a, 0xc3];
    let mut received = expected;
    let passed = spi.transfer_in_place(&mut received).is_ok() && received == expected;
    let report = if passed { PASS_REPORT } else { FAIL_REPORT };
    let mut report_offset = 0_usize;
    let mut terminal_was_open = false;
    let mut elapsed_ms = 0_u16;

    loop {
        console.poll();
        let terminal_open = console.is_terminal_open();
        if terminal_open && !terminal_was_open {
            report_offset = 0;
        }
        terminal_was_open = terminal_open;

        if terminal_open && report_offset < report.len() {
            match console.write(&report[report_offset..]) {
                Ok(count) => report_offset += count,
                Err(Error::WouldBlock) => {}
                Err(_) => panic!(),
            }
        }

        delay.delay_ms(1).unwrap_or_else(|_| panic!());
        elapsed_ms += 1;
        if elapsed_ms >= 500 {
            led.toggle().unwrap_or_else(|_| panic!());
            elapsed_ms = 0;
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
