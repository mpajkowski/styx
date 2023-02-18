use alloc::vec;

use crate::x86_64::heap;
use crate::x86_64::interrupts;
use crate::x86_64::limine::Limine;

use super::drivers;
use super::gdt;
use super::logger;
use super::pmm;

#[no_mangle]
pub extern "C" fn _x86_64_bsp_entrypoint() {
    super::sync::disable_interrupts();

    let boot_info = Limine::gather();

    let com1 = drivers::Serial::init_com1().unwrap();
    let framebuffer = boot_info.framebuffer();
    let terminal = crate::Terminal::new(framebuffer.width(), framebuffer.height());
    framebuffer.install();

    logger::initialize(com1, terminal);
    log::info!("Installed logger");

    log::info!("Installing early GDT");
    gdt::early_init();

    log::info!("Installing interrupts");
    interrupts::init();

    pmm::initialize(&boot_info);
    heap::initialize();

    let mut vec = vec![1, 2, 3];
    vec[0] = 5;
    log::info!("Initialized heap: {vec:?}");

    log::info!("Through the jungle by the river Styx");
    log::info!("I've journed long and far this day");

    super::sync::enable_interrupts();

    crate::main();
}
