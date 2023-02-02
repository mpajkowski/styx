use crate::x86_64::limine::Limine;

use super::drivers;
use super::gdt;
use super::logger;

#[no_mangle]
pub extern "C" fn _x86_64_bsp_entrypoint() {
    super::sync::disable_interrupts();

    let _boot_info = Limine::gather();
    let com1 = drivers::Serial::init_com1().unwrap();

    logger::initialize(com1);

    log::info!("Loading early GDT");
    gdt::early_init();

    log::info!("Through the jungle by the river Styx");
    log::info!("I've journed long and far this day");

    crate::main();
}
