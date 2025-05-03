#![no_std]
#![no_main]
use core::{arch::asm, panic::PanicInfo};
anyhal_stm32h563::stm32h563_runtime!();
fn firmware_main() -> ! {
    loop {
        unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) };
    }
}
#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
