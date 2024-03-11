use crate::arch::ap;
use crate::arch::cpulocal;
use crate::arch::features;
use crate::arch::interrupts::lapic;
use crate::arch::interrupts::pic;
use crate::arch::kernel_elf;
use crate::arch::modules::Modules;
use crate::arch::sync::hlt;
use crate::arch::VirtAddr;
use crate::x86_64::drivers::pit;
use crate::x86_64::drivers::ps2;
use crate::x86_64::heap;
use crate::x86_64::interrupts;
use crate::x86_64::limine::Limine;
use crate::Framebuffer;

use super::acpi;
use super::drivers;
use super::logger;
use super::pmm;
use super::segmentation;

#[no_mangle]
pub extern "C" fn _x86_64_bsp_entrypoint() {
    super::sync::disable_interrupts();

    let boot_info = Limine::gather();

    let cmdline = boot_info.kernel.cmdline();
    let config = config::Config::from_cmdline(cmdline);

    kernel_elf::from_boot_info(&boot_info);

    let com1 = drivers::Serial::init_com1().unwrap();
    let framebuffer = boot_info.framebuffer();
    let terminal = crate::Terminal::new(framebuffer.width(), framebuffer.height());
    framebuffer.install();

    logger::initialize(com1, terminal, &config);
    log::info!("Installed logger");
    log::info!("cmdline: {:?}", config.cmdline);

    let (features, ext_features) = features::init();

    log::info!("Installing early GDT");
    segmentation::early_init(&ext_features);

    log::info!("Installing interrupts");
    interrupts::init();
    pic::remap_and_disable();

    pmm::initialize(&boot_info);
    heap::initialize();

    lapic::init(&features);

    acpi::init(&boot_info).expect("failed to initialize apci tables");

    ap::start_aps();

    // use new stack
    let stack = heap::alloc_stack();
    unsafe { core::arch::asm!("mov rsp, {}", in(reg) stack.to_u64()) };

    cpulocal::init(interrupts::lapic::local_apic().bsp_id() as u64, stack);

    ap::set_bsp_ready();

    log::info!("Through the jungle by the river Styx");
    log::info!("I've journed long and far this day");

    log::info!("BSP ready");
    let modules = Modules::from_boot_info(boot_info.module.modules());
    if let Some(rudzik) = modules.by_path("/rudzik.data") {
        logger::disable_terminal();
        Framebuffer::with_handle_mut(|fb| fb.put_bitmap(357, rudzik));
    } else {
        log::warn!(
            "Rudzik was not loaded, I wanted to panic but decided not to do so, but beware!"
        );
    }

    ps2::init();
    pit::init();

    super::sync::enable_interrupts();

    crate::main();
}

#[no_mangle]
pub extern "C" fn _x86_64_ap_entrypoint(ap_id: u64, stack_top_addr: VirtAddr) {
    let (_features, ext_features) = features::init();
    segmentation::early_init(&ext_features);
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
