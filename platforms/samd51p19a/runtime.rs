/// Installs the SAMD51 vector table, reset initialization, and fault handlers.
/// The invoking binary must define `fn firmware_main() -> !`.
#[macro_export]
macro_rules! samd51_runtime {
    () => {
        core::arch::global_asm!(
            r#"
            .syntax unified
            .cpu cortex-m4
            .thumb
            .section .vectors, "a", %progbits
            .global VECTOR_TABLE
        VECTOR_TABLE:
            .word _estack
            .word Reset
            .rept 14
            .word DefaultHandler
            .endr
        "#
        );

        extern "C" {
            static mut _sidata: u32;
            static mut _sdata: u32;
            static mut _edata: u32;
            static mut _sbss: u32;
            static mut _ebss: u32;
        }

        #[no_mangle]
        unsafe extern "C" fn Reset() -> ! {
            let mut source = core::ptr::addr_of!(_sidata);
            let mut destination = core::ptr::addr_of_mut!(_sdata);
            let data_end = core::ptr::addr_of_mut!(_edata);
            while destination < data_end {
                destination.write(source.read());
                destination = destination.add(1);
                source = source.add(1);
            }
            let mut bss = core::ptr::addr_of_mut!(_sbss);
            let bss_end = core::ptr::addr_of_mut!(_ebss);
            while bss < bss_end {
                bss.write(0);
                bss = bss.add(1);
            }
            firmware_main()
        }

        #[no_mangle]
        extern "C" fn DefaultHandler() -> ! {
            loop {
                unsafe { core::arch::asm!("wfi", options(nomem, nostack, preserves_flags)) };
            }
        }
    };
}
