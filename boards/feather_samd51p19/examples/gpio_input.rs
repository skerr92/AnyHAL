#![no_std]
#![no_main]

use anyhal::hal::gpio::{InputPin, OutputPin, Pull};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

fn firmware_main() -> ! {
    unsafe { anyhal_samd51p19a::configure_dfll48m().unwrap_or_else(|_| panic!()) };
    let button =
        unsafe { anyhal_samd51p19a::InputPin::claim(anyhal_board_feather_samd51p19::A0, Pull::Up) };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };

    loop {
        if button.is_low() {
            led.set_high().unwrap_or_else(|_| panic!());
        } else {
            led.set_low().unwrap_or_else(|_| panic!());
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
