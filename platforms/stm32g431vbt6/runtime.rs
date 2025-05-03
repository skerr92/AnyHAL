#[macro_export]
macro_rules! stm32g431vbt6_runtime {
    () => {
        core::arch::global_asm!(
            r#".syntax unified
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
            let mut s = core::ptr::addr_of!(_sidata);
            let mut d = core::ptr::addr_of_mut!(_sdata);
            let de = core::ptr::addr_of_mut!(_edata);
            while d < de {
                d.write(s.read());
                d = d.add(1);
                s = s.add(1);
            }
            let mut b = core::ptr::addr_of_mut!(_sbss);
            let be = core::ptr::addr_of_mut!(_ebss);
            while b < be {
                b.write(0);
                b = b.add(1);
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
