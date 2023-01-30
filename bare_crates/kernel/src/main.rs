#![no_std]
#![no_main]

mod limine;
mod logger;
mod panic;

pub mod drivers;
pub mod sync;

use core::hint::spin_loop;

pub use crate::limine::Limine;

#[no_mangle]
extern "C" fn _start() {
    sync::disable_interrupts();

    let boot_info = Limine::gather();
    let term = drivers::Terminal::from_boot_info(&boot_info);
    let com1 = drivers::Serial::init_com1().unwrap();

    logger::initialize(term, com1);

    log::info!("Through the jungle by the river Styx");
    log::info!("I've journed long and far this day");

    loop {
        spin_loop();
    }
}
