use raw_cpuid::ExtendedFeatures;

use crate::arch::registers::Cr4;

static mut READ_FS: unsafe fn() -> u64 = impl_msr::read_fs;
static mut READ_GS: unsafe fn() -> u64 = impl_msr::read_gs;
static mut WRITE_FS: unsafe fn(u64) = impl_msr::write_fs;
static mut WRITE_GS: unsafe fn(u64) = impl_msr::write_gs;

pub fn init(feat: &ExtendedFeatures) {
    if feat.has_fsgsbase() {
        log::info!("using FSGSBASE implementation");

        Cr4::read().union(Cr4::FSGSBASE).write();

        log::debug!("Cr4: {:?}", Cr4::read());

        unsafe {
            READ_FS = impl_fsgsbase::read_fs;
            READ_GS = impl_fsgsbase::read_gs;
            WRITE_FS = impl_fsgsbase::write_fs;
            WRITE_GS = impl_fsgsbase::write_gs;
        }
    } else {
        log::info!("using MSR FS/GS implementation");
    }
}

#[inline(always)]
pub fn read_fs() -> u64 {
    unsafe { READ_FS() }
}

#[inline(always)]
pub fn read_gs() -> u64 {
    unsafe { READ_GS() }
}

#[inline(always)]
pub fn write_fs(val: u64) {
    unsafe { WRITE_FS(val) }
}

#[inline(always)]
pub fn write_gs(val: u64) {
    unsafe { WRITE_GS(val) }
}

mod impl_msr {
    use crate::x86_64::msr;

    const IA32_FS_BASE: u32 = 0xc0000100;
    const IA32_GS_BASE: u32 = 0xc0000101;
    //const IA32_KERNEL_GS_BASE: u32 = 0xc0000102;

    pub unsafe fn read_fs() -> u64 {
        msr::rdmsr(IA32_FS_BASE)
    }

    pub unsafe fn read_gs() -> u64 {
        msr::rdmsr(IA32_GS_BASE)
    }

    pub unsafe fn write_fs(val: u64) {
        msr::wrmsr(IA32_FS_BASE, val)
    }

    pub unsafe fn write_gs(val: u64) {
        msr::wrmsr(IA32_GS_BASE, val)
    }
}

mod impl_fsgsbase {
    use core::arch::asm;

    pub unsafe fn read_fs() -> u64 {
        let val: u64;
        asm!("rdfsbase {}", out(reg) val);
        val
    }

    pub unsafe fn read_gs() -> u64 {
        let val: u64;
        asm!("rdfsbase {}", out(reg) val);
        val
    }

    pub unsafe fn write_fs(val: u64) {
        asm!("wrfsbase {}", in(reg) val);
    }

    pub unsafe fn write_gs(val: u64) {
        asm!("wrgsbase {}", in(reg) val);
    }
}
