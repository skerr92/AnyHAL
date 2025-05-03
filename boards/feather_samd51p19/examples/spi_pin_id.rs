#![no_std]
#![no_main]

use anyhal::hal::{
    delay::DelayMs,
    gpio::{InputPin, OutputPin, Pin, Port, Pull},
};
use core::{arch::asm, panic::PanicInfo};

anyhal_samd51p19a::samd51_runtime!();

const SAMPLE_MS: u32 = 10;
const DEBOUNCE_SAMPLES: u8 = 3;

fn firmware_main() -> ! {
    let clocks = unsafe { anyhal_samd51p19a::configure_dfll48m().unwrap_or_else(|_| panic!()) };
    let pb12 = unsafe { anyhal_samd51p19a::InputPin::claim(Pin::new(Port::B, 12), Pull::Up) };
    let pb13 = unsafe { anyhal_samd51p19a::InputPin::claim(Pin::new(Port::B, 13), Pull::Down) };
    let pb14 = unsafe { anyhal_samd51p19a::InputPin::claim(Pin::new(Port::B, 14), Pull::Down) };
    let mut led =
        unsafe { anyhal_samd51p19a::OutputPin::claim(anyhal_board_feather_samd51p19::LED, false) };
    let mut delay = unsafe {
        anyhal_samd51p19a::SysTickDelay::claim(clocks.core_hz()).unwrap_or_else(|_| panic!())
    };

    let mut latched_led = false;
    let mut pb12_was_low = false;
    let mut pb12_low_samples = 0_u8;
    let mut blink_elapsed_ms = 0_u32;
    let mut active_period_ms = 0_u32;

    loop {
        if pb12.is_low() {
            pb12_low_samples = pb12_low_samples.saturating_add(1);
            if pb12_low_samples >= DEBOUNCE_SAMPLES && !pb12_was_low {
                latched_led = !latched_led;
                pb12_was_low = true;
            }
        } else {
            pb12_low_samples = 0;
            pb12_was_low = false;
        }

        let requested_period_ms = if pb13.is_high() {
            500
        } else if pb14.is_high() {
            1_000
        } else {
            0
        };

        if requested_period_ms == 0 {
            active_period_ms = 0;
            blink_elapsed_ms = 0;
            if latched_led {
                led.set_high().unwrap_or_else(|_| panic!());
            } else {
                led.set_low().unwrap_or_else(|_| panic!());
            }
        } else {
            if requested_period_ms != active_period_ms {
                active_period_ms = requested_period_ms;
                blink_elapsed_ms = 0;
                led.set_high().unwrap_or_else(|_| panic!());
            }
            blink_elapsed_ms += SAMPLE_MS;
            if blink_elapsed_ms >= active_period_ms {
                led.toggle().unwrap_or_else(|_| panic!());
                blink_elapsed_ms = 0;
            }
        }

        delay.delay_ms(SAMPLE_MS).unwrap_or_else(|_| panic!());
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("bkpt", options(nomem, nostack, preserves_flags)) };
    }
}
