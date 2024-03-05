use core::arch::asm;

use bitflags::bitflags;

use crate::{arch::FRAME_SIZE, x86_64::PhysAddr};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScratchRegisters {
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PreservedRegisters {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IretRegisters {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

macro_rules! impl_cr {
    ($name:ident, $num:expr) => {
        impl $name {
            /// Read value from register
            pub fn read() -> Self {
                let val: u64;
                unsafe { asm!(concat!("mov {}, cr", $num), out(reg) val, options(nomem, nostack, preserves_flags)) };
                Self::from_bits_truncate(val)
            }

            /// Write value to register
            pub fn write(self)  {
                unsafe { asm!(concat!("mov cr", $num, ", {}"), in(reg) self.bits()) };
            }
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Cr0: u64 {
        const PE = 1;
        const MP = 1 << 1;
        const EM = 1 << 2;
        const TS = 1 << 3;
        const ET = 1 << 4;
        const NE = 1 << 5;
        // 6-15 reserved
        const WP = 1 << 16;
        // 17 reserved
        const AM = 1 << 18;
        // 19-28 reserved
        const NW = 1 << 29;
        const CD = 1 << 30;
        const PG = 1 << 31;
        // 32-63 reserved
    }
}

impl_cr!(Cr0, 0);

#[derive(Debug, Clone, Copy)]
pub struct Cr3(PhysAddr);

impl Cr3 {
    pub fn read() -> Self {
        let val: u64;

        unsafe {
            asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags));
        }

        Self(PhysAddr::new_aligned::<FRAME_SIZE>(
            val & 0x_000f_ffff_ffff_f000,
        ))
    }

    pub fn phys_addr(self) -> PhysAddr {
        self.0
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Cr4: u64 {
        const VME = 1;
        const PVI = 1 << 1;
        const TSD = 1 << 2;
        const DE  = 1 << 3;
        const PSE = 1 << 4;
        const PAE = 1 << 5;
        const MCE = 1 << 6;
        const PGE = 1 << 7;
        const PCE = 1 << 8;
        const OSFXSR = 1 << 9;
        const OSXMMEXCPT = 1 << 10;
        const UIMP = 1 << 11;
        // 12 reserved
        const VMXE = 1 << 13;
        const SMXE = 1 << 14;
        const RES15 = 1 << 15;
        const FSGSBASE = 1 << 16;
        const PCIDE = 1 << 17;
        const OSXSAVE = 1 << 18;
        // 19 reserved
        const SMEP = 1 << 20;
        const SMAP = 1 << 21;
        const PKE = 1 << 22;
        const CET = 1 << 23;
        const PKS = 1 << 24;
        // 25-63 reserved
    }
}

impl_cr!(Cr4, 4);
