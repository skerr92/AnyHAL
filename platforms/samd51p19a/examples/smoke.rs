#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

fn firmware_main() -> ! {
    let _version = anyhal::VERSION;
    loop {
        unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
