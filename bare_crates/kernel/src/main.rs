#![no_std]
#![no_main]

mod panic;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub mod arch {
    pub use super::x86_64::*;
}

pub fn main() {
    log::info!("Initialized architecture");

    // test if page fault occurrs
    unsafe {
        let x: u32 = core::ptr::read_volatile(0x0 as *const _);
        log::info!("X: {x}");
    }

    loop {
        arch::sync::hlt()
    }
}
