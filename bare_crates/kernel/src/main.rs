#![no_std]
#![no_main]

mod drivers;
mod panic;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub mod arch {
    pub use super::x86_64::*;
}

pub use drivers::Framebuffer;
pub use drivers::Terminal;

pub fn main() {
    log::info!("Initialized architecture");

    loop {
        arch::sync::hlt()
    }
}
