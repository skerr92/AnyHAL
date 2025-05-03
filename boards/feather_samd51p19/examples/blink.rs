#![no_std]
#![no_main]

use anyhal::hal::{delay::DelayMs, gpio::OutputPin};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

fn firmware_main() -> ! {
    let clocks = unsafe { anyhal_samd51p19a::configure_dfll48m().unwrap_or_else(|_| panic!()) };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };
    loop {
        led.toggle().unwrap_or_else(|_| panic!());
        delay.delay_ms(500).unwrap_or_else(|_| panic!());
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
