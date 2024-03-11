#![no_std]
#![no_main]
#![allow(clippy::bad_bit_mask)]

extern crate alloc;

mod drivers;
mod panic;

pub mod kernel_elf;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub mod arch {
    pub use super::x86_64::*;
}

pub use drivers::Framebuffer;
pub use drivers::Terminal;

#[global_allocator]
pub static HEAP_ALLOC: arch::HeapAllocator = arch::HeapAllocator::uninitialized();

pub fn main() {
    log::info!("Initialized architecture");

    loop {
        arch::sync::hlt()
    }
}
