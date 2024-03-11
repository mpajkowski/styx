use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
#[inline(never)]
fn rust_begin_unwind(info: &PanicInfo) -> ! {
    crate::arch::sync::disable_interrupts();

    log::error!("KERNEL PANIC: {info}");

    crate::arch::unwind();

    loop {
        crate::arch::sync::hlt();
    }
}
