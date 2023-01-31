use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("KERNEL PANIC: {info:?}");

    loop {
        crate::arch::sync::hlt();
    }
}
