use crate::arch::ap;
use crate::arch::cpulocal;
use crate::arch::features;
use crate::arch::fsgs;
use crate::arch::interrupts::lapic;
use crate::arch::sync::hlt;
use crate::arch::VirtAddr;
use crate::x86_64::heap;
use crate::x86_64::interrupts;
use crate::x86_64::limine::Limine;

use super::acpi;
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

    let (features, _ext_features) = features::init();

    log::info!("Installing early GDT");
    gdt::early_init();

    log::info!("Installing interrupts");
    interrupts::init();

    interrupts::legacy_pic::init();

    pmm::initialize(&boot_info);
    heap::initialize();

    lapic::init(&features);

    acpi::init(&boot_info).expect("failed to initialize apci tables");

    ap::start_aps();

    // use new stack
    let stack = heap::alloc_stack();
    unsafe { core::arch::asm!("mov rsp, {}", in(reg) stack.to_u64()) };

    cpulocal::init(interrupts::lapic::local_apic().bsp_id() as u64, stack);

    log::info!("Through the jungle by the river Styx");
    log::info!("I've journed long and far this day");

    ap::set_bsp_ready();

    super::sync::enable_interrupts();

    log::info!("BSP ready");

    crate::main();
}

#[no_mangle]
pub extern "C" fn _x86_64_ap_entrypoint(ap_id: u64, stack_top_addr: VirtAddr) {
    gdt::early_init();
    interrupts::init_ap();
    cpulocal::init(ap_id, stack_top_addr);

    log::info!("AP {ap_id} ready, waiting for bsp");
    ap::notify_booted(ap_id);
    ap::wait_for_bsp();
    log::info!("AP {ap_id} successfully initialized");

    loop {
        hlt();
    }
}
