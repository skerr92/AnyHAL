#![no_std]
#![no_main]

use anyhal::hal::{delay::DelayMs, gpio::OutputPin, i2c::I2cBus};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

fn firmware_main() -> ! {
    let clocks = unsafe { anyhal_samd51p19a::configure_dfll48m().unwrap_or_else(|_| panic!()) };
    let mut bus = unsafe {
        anyhal_samd51p19a::Sercom1I2c::claim(
            anyhal_board_feather_samd51p19::SDA,
            anyhal_board_feather_samd51p19::SCL,
            clocks,
            100_000,
        )
        .unwrap_or_else(|_| panic!())
    };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };

    let mut found = false;
    for address in 0x08..=0x77 {
        if bus.write(address, &[]).is_ok() {
            found = true;
            break;
        }
    }

    loop {
        if found {
            led.set_high().unwrap_or_else(|_| panic!());
            delay.delay_ms(1_000).unwrap_or_else(|_| panic!());
        } else {
            led.toggle().unwrap_or_else(|_| panic!());
            delay.delay_ms(250).unwrap_or_else(|_| panic!());
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
