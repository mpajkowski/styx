use core::{arch::asm, fmt::Display};

use crate::arch::{
    paging::{address_space::AddressSpace, offset_table::TranslateResult},
    VirtAddr,
};

use super::cpulocal;

#[repr(C)]
struct StackFrame {
    rbp: *mut StackFrame,
    rip: u64,
}

#[inline(always)]
pub fn unwind() {
    let mut stack_frame: *mut StackFrame;

    unsafe {
        asm!("mov {}, rbp", out(reg) stack_frame);
    }

    let cpu = cpulocal::CpuLocal::obtain()
        .map(|c| c.info.lapic_id)
        .unwrap_or(0);

    log::error!("Occurred on core {cpu}");

    let Some(elf) = crate::kernel_elf::get() else {
        log::error!("no debug info");
        return;
    };

    let mut address_space = AddressSpace::active();
    let offset_table = address_space.offset_table();

    let Some(symtable) = elf.symtable() else {
        log::error!("no symtable");
        return;
    };

    log::error!("Backtrace:");
    for depth in 0..128 {
        let rip = if !stack_frame.is_null() {
            unsafe { (*stack_frame).rip }
        } else {
            break;
        };

        if rip == 0 {
            break;
        }

        let rip = match VirtAddr::try_new(rip) {
            Ok(addr) => addr,
            Err(err) => {
                log::error!("invalid addr: {err}");
                break;
            }
        };

        if matches!(
            offset_table.translate(rip),
            TranslateResult::NotMapped | TranslateResult::InvalidFrameAddress(_)
        ) {
            log::error!("unmapped rip");
            break;
        }

        stack_frame = unsafe { (*stack_frame).rbp };

        let rip = rip.to_u64();
        let symbol_name = elf.symbol_name_at_addr(symtable, rip);

        let symbol_name: &dyn Display = symbol_name
            .as_ref()
            .map(|sym| sym as _)
            .unwrap_or(&"unknown");

        log::error!("{depth:>2} {rip:x} [{symbol_name}]");
    }

    log::error!("Backtrace end");
}
