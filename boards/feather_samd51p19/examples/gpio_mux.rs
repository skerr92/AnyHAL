#![no_std]
#![no_main]

use anyhal::hal::{
    delay::DelayMs,
    gpio::{AlternateConfig, AlternateFunction, OutputPin, OutputType, Pull, Speed},
};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

fn firmware_main() -> ! {
    let clocks = unsafe { anyhal_samd51p19a::configure_dfll48m().unwrap_or_else(|_| panic!()) };
    let config = AlternateConfig {
        // PB13 function C is SERCOM4 PAD1, used by the board SCK alias.
        function: AlternateFunction::new(2).unwrap_or_else(|| panic!()),
        pull: Pull::None,
        output_type: OutputType::PushPull,
        speed: Speed::VeryHigh,
    };
    let _sck = unsafe {
        anyhal_samd51p19a::AlternatePin::claim(anyhal_board_feather_samd51p19::SCK, config)
            .unwrap_or_else(|_| panic!())
    };
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
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
