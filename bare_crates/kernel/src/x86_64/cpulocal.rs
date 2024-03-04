use crate::arch::VirtAddr;

use super::{
    heap,
    segmentation::{self, write_gs, GdtEntry, Tss},
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
        let ptr = segmentation::read_gs();

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

    segmentation::late_init(stack, unsafe { &mut *cpulocal });
    write_gs(cpulocal as u64);
}
