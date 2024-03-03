use core::{
    arch::asm,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use spin::Once;

use acpi::platform::{ProcessorInfo, ProcessorState};

use crate::{
    arch::{interrupts::lapic, paging::address_space::read_cr3_addr, VirtAddr},
    x86_64::{heap::alloc_stack, ioport::delay},
};

use super::interrupts::lapic::LocalApic;

pub static PROCESSOR_INFO: Once<ProcessorInfo> = Once::new();

const CPU_SLICE_BEGIN: *mut bool = 0x3000 as _;
static BSP_READY: AtomicBool = AtomicBool::new(false);

extern "C" {
    /// Loads trampoline into conventional memory
    fn load_trampoline() -> u16;

    /// Sets arguments for AP
    ///
    /// # Arguments:
    ///
    ///   * `page_table` - current value of CR3 register
    ///   * `stack_top` - stack allocated for this CPU
    ///   * `boot_info`- limine boot info struct
    ///   * `ap_id`- AP ID read from LocalApic entry
    fn prepare_ap_launch(page_table: u64, stack_top: VirtAddr, ap_id: u8);

    /// Returns trampoline size in bytes
    fn trampoline_size() -> usize;

    /// Checks AP flag - if `true` then we can assume that AP boot has succeed
    fn is_ap_ready() -> bool;
}

pub fn start_aps() {
    let Some(processors) = PROCESSOR_INFO.get() else {
        log::warn!("no processor info found");
        return;
    };

    if processors.application_processors.is_empty() {
        log::info!("no APs found");
        return;
    }

    let page_idx = unsafe { load_trampoline() };
    log::info!(
        "trampoline of size {} loaded, page_idx: {page_idx}",
        unsafe { trampoline_size() }
    );

    let mut bsp = lapic::local_apic();

    let cpus = unsafe {
        core::slice::from_raw_parts_mut(CPU_SLICE_BEGIN, processors.application_processors.len())
    };

    cpus.fill(false);

    cpus[processors.boot_processor.local_apic_id as usize] = true;

    for processor in processors.application_processors.iter() {
        if !processor.is_ap || processor.state == ProcessorState::Disabled {
            cpus[processor.local_apic_id as usize] = true;
            continue;
        }

        let lapic_id = processor.local_apic_id as u8;

        log::info!("APIC ID: {lapic_id}");
        boot_ap(lapic_id, &mut bsp)
    }

    // wait for APs
    unsafe {
        while !cpus.iter().all(|c| ptr::read_volatile(c)) {
            asm!("pause", options(nomem, nostack));
        }
    }
}

fn boot_ap(apic_id: u8, bsp: &mut LocalApic) {
    log::info!("Booting APIC ID: {apic_id}...");

    log::trace!("reserving stack");
    let ap_stack = alloc_stack();

    log::trace!("preparing launch");
    unsafe { prepare_ap_launch(read_cr3_addr().to_u64(), ap_stack, apic_id) };

    log::trace!("prepared launch");

    // init IPI...
    bsp.send_init_ipi(apic_id as u64);
    log::trace!("after init ipi");
    delay(5000);

    // startup IPI..
    bsp.send_startup_ipi(apic_id as u64);
    log::trace!("after startup ipi");

    unsafe {
        while !is_ap_ready() {
            asm!("pause", options(nomem, nostack));
            delay(5000);
        }
    }
}

/// AP marks itself as booted
pub fn notify_booted(lapic_idx: u64) {
    unsafe { CPU_SLICE_BEGIN.add(lapic_idx as usize).write_volatile(true) };
}

/// BSP marks itself as ready; APs must wait using `wait_for_bsp`
pub fn set_bsp_ready() {
    BSP_READY.store(true, Ordering::SeqCst);
}

/// Waits until BSP calls `set_bsp_ready`. To be called by AP.
pub fn wait_for_bsp() {
    while !BSP_READY.load(Ordering::SeqCst) {
        core::hint::spin_loop()
    }
}
