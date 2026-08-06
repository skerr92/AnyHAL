#![no_std]
#![no_main]

use anyhal::hal::{delay::DelayMs, gpio::OutputPin, pwm::PwmOutput, serial::SerialConsole, Error};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

const REPORT: &[u8] = b"\r\nAnyHAL PWM test\r\n\
[PASS] TC4 match-PWM configured at 20 kHz\r\n\
[PASS] output routed to board A1 / PB09 / TC4-WO1\r\n\
[RUN] duty cycle is sweeping between 0% and 100%\r\n\
Connect an LED with a series resistor from A1 to GND, or probe A1.\r\n";

fn firmware_main() -> ! {
    let (mut console, clocks) =
        unsafe { anyhal_samd51p19a::UsbCdcConsole::claim().unwrap_or_else(|_| panic!()) };
    let mut pwm = unsafe {
        anyhal_samd51p19a::Tc4Pwm::claim(anyhal_board_feather_samd51p19::A1, clocks, 20_000)
            .unwrap_or_else(|_| panic!())
    };
    let mut status =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };

    pwm.enable().unwrap_or_else(|_| panic!());
    let mut report_offset = 0;
    let mut terminal_was_open = false;
    let mut duty = 0_u16;
    let mut rising = true;

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
        }

        pwm.set_duty_cycle(duty).unwrap_or_else(|_| panic!());
        if rising {
            let next = duty.saturating_add(512);
            if next == u16::MAX {
                rising = false;
                status.toggle().unwrap_or_else(|_| panic!());
            }
            duty = next;
        } else {
            let next = duty.saturating_sub(512);
            if next == 0 {
                rising = true;
                status.toggle().unwrap_or_else(|_| panic!());
            }
            duty = next;
        }
        delay.delay_ms(8).unwrap_or_else(|_| panic!());
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
