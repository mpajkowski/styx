use crate::x86_64::limine::Limine;

use super::drivers;
use super::logger;

#[no_mangle]
pub extern "C" fn _x86_64_bsp_entrypoint() {
    super::sync::disable_interrupts();

    let boot_info = Limine::gather();
    let term = drivers::Terminal::from_boot_info(&boot_info);
    let com1 = drivers::Serial::init_com1().unwrap();

    logger::initialize(term, com1);

    log::info!("Through the jungle by the river Styx");
    log::info!("I've journed long and far this day");

    crate::main();
}
