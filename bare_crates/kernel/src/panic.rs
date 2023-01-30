use core::{hint::spin_loop, panic::PanicInfo};

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {
        spin_loop();
    }
}
