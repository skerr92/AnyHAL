#![no_std]
#![no_main]
use anyhal::hal::{delay::DelayMs, gpio::OutputPin};
use core::{arch::asm, panic::PanicInfo};
anyhal_stm32h563::stm32h563_runtime!();
fn firmware_main() -> ! {
    let mut led =
        unsafe { anyhal_stm32h563::OutputPin::claim(anyhal_board_nucleo_h563zi::LED, false) };
    let mut delay = unsafe {
        anyhal_stm32h563::SysTickDelay::claim(anyhal_stm32h563::RESET_CORE_HZ)
            .unwrap_or_else(|_| panic!())
    };
    loop {
        led.toggle().unwrap_or_else(|_| panic!());
        delay.delay_ms(500).unwrap_or_else(|_| panic!());
    }
}
#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
