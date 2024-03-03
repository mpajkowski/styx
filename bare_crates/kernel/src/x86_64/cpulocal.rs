use crate::arch::VirtAddr;

use super::{
    fsgs,
    gdt::{self, GdtEntry, Tss},
    heap,
};

/// Stores per-cpu local data
#[repr(C)]
#[derive(Debug)]
pub struct CpuLocal {
    pub tss: Tss,
    pub info: &'static mut CpuInfo,
}

impl CpuLocal {
    pub fn obtain() -> Option<&'static mut Self> {
        let ptr = fsgs::read_gs();

        if ptr == 0 {
            return None;
        }

        Some(unsafe { &mut *(ptr as *mut Self) })
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CpuInfo {
    pub lapic_id: u64,
    pub gdt: &'static mut [GdtEntry],
}

pub fn init(lapic_id: u64, stack: VirtAddr) {
    let cpulocal = heap::alloc::<CpuLocal>();
    let cpuinfo = heap::alloc::<CpuInfo>();

    unsafe {
        (*cpuinfo).lapic_id = lapic_id;
        (*cpulocal).info = &mut *cpuinfo;
    }

    gdt::late_init(stack, cpulocal);

    fsgs::write_gs(cpulocal as u64);
}
