#![allow(unused)]

use core::{arch::asm, fmt::Debug, mem::size_of};

use crate::arch::VirtAddr;

use super::DescriptorPointer;

bitflags::bitflags! {
    pub struct GdtEntryFlags: u8 {
        const NULL = 0;
        const PROTECTED_MODE = 1 << 6;
        const LONG_MODE = 1 << 5;
    }
}

bitflags::bitflags! {
    pub struct GdtAccessFlags: u8 {
        const NULL = 0;
        const PRESENT = 1 << 7;
        const RING_0 = 0 << 5;
        const RING_3 = 3 << 5;
        const SYSTEM = 1 << 4;
        const EXECUTABLE = 1 << 3;
        const PRIVILEGE = 1 << 1;
        const TSS_AVAIL = 9;
    }
}

#[repr(u16)]
pub enum GdtEntryType {
    KernelCode = 1,
    KernelData = 2,
    KernelTls = 3,
    TssLow = 8,
    TssHi = 9,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Ring {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

#[repr(C, packed)]
pub struct Tss {
    reserved: u32,
    pub rsp: [VirtAddr; 3],
    reserved_0: u64,
    pub ist: [VirtAddr; 7],
    reserved_1: u32,
    reserved_2: u32,
    reserved_3: u16,
    iopb_offset: u16,
}

impl Default for Tss {
    fn default() -> Self {
        Self {
            reserved: 0,
            rsp: [VirtAddr::zero(); 3],
            reserved_0: 0,
            ist: [VirtAddr::zero(); 7],
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            iopb_offset: 1,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct GdtEntry {
    pub limit0_15: u16,
    pub base0_15: u16,
    pub base16_23: u8,
    pub flags: u8,
    pub limit16_19_flags: u8,
    pub base24_31: u8,
}

impl GdtEntry {
    pub const NULL: Self = Self::new(GdtAccessFlags::NULL, GdtEntryFlags::NULL);

    pub const fn new(access_flags: GdtAccessFlags, entry_flags: GdtEntryFlags) -> Self {
        Self {
            limit0_15: 0x00,
            base0_15: 0x00,
            base16_23: 0x00,
            flags: access_flags.bits(),
            limit16_19_flags: entry_flags.bits() & 0xf0,
            base24_31: 0x00,
        }
    }

    pub fn set_limit(&mut self, limit: u32) {
        self.limit0_15 = limit as u16;
        self.limit16_19_flags = self.limit16_19_flags & 0xf0 | (limit >> 16) as u8;
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.base0_15 = offset as u16;
        self.base16_23 = (offset >> 16) as u8;
        self.base24_31 = (offset >> 24) as u8;
    }

    pub fn set_raw<T>(&mut self, raw: T) {
        let selfptr = self as *mut _ as *mut T;
        unsafe { *selfptr = raw }
    }
}

/// Loads GDT using `lgdt`
///
/// ## Safety
///
/// It is unsafe as hell, just be warned.
///
/// NOTE the `&mut` - enforces that GDT table is placed within writable region
#[inline(always)]
unsafe fn load(table: &'static mut [GdtEntry]) {
    let ptr = DescriptorPointer {
        size: (table.len() * size_of::<GdtEntry>() - 1) as u16,
        address: VirtAddr::new(table.as_ptr() as u64),
    };

    asm!("lgdt [{}]", in(reg) &ptr, options(nostack));
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SegmentSelector(pub(crate) u16);

impl Debug for SegmentSelector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SegmentSelector")
            .field("index", &(self.0 >> 3))
            .field(
                "ring",
                &match (self.0 as u8) & 3 {
                    0 => Ring::Ring0,
                    1 => Ring::Ring1,
                    2 => Ring::Ring2,
                    3 => Ring::Ring3,
                    _ => unreachable!(),
                },
            )
            .finish()
    }
}

impl SegmentSelector {
    pub fn new(index: GdtEntryType, ring: Ring) -> Self {
        Self((index as u16) << 3 | (ring as u16))
    }

    pub const fn zeroed() -> Self {
        Self(0)
    }

    pub const fn from_raw(raw: u16) -> Self {
        SegmentSelector(raw)
    }
}

impl From<u16> for SegmentSelector {
    fn from(x: u16) -> Self {
        Self::from_raw(x)
    }
}

#[inline(always)]
unsafe fn load_cs(selector: SegmentSelector) {
    /*
     * NOTE: We cannot directly move into CS since x86 requires the IP
     * and CS set at the same time. To do this, we need push the new segment
     * selector and return value onto the stack and far return to reload CS and
     * continue execution.
     *
     * We also cannot use a far call or a far jump since we would only be
     * able to jump to 32-bit instruction pointers. Only Intel supports for
     * 64-bit far calls/jumps in long-mode, AMD does not.
     */
    asm!(
        "push {selector}",
        "lea {tmp}, [1f + rip]",
        "push {tmp}",
        "retfq",
        "1:",
        selector = in(reg) u64::from(selector.0),
        tmp = lateout(reg) _,
    );
}

#[inline(always)]
unsafe fn load_ds(selector: SegmentSelector) {
    asm!("mov ds, {0:x}", in(reg) selector.0, options(nomem, nostack))
}

pub unsafe fn get_cs() -> SegmentSelector {
    let segment: u16;
    asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));

    SegmentSelector(segment)
}

#[inline(always)]
unsafe fn load_es(selector: SegmentSelector) {
    asm!("mov es, {0:x}", in(reg) selector.0, options(nomem, nostack))
}

#[inline(always)]
unsafe fn load_fs(selector: SegmentSelector) {
    asm!("mov fs, {0:x}", in(reg) selector.0, options(nomem, nostack))
}

#[inline(always)]
unsafe fn load_gs(selector: SegmentSelector) {
    asm!("mov gs, {0:x}", in(reg) selector.0, options(nomem, nostack))
}

#[inline(always)]
unsafe fn load_ss(selector: SegmentSelector) {
    asm!("mov ss, {0:x}", in(reg) selector.0, options(nomem, nostack))
}

#[inline(always)]
unsafe fn load_tss(selector: SegmentSelector) {
    asm!("ltr {0:x}", in(reg) selector.0, options(nostack, nomem));
}

static mut GDT_BOOT: [GdtEntry; 3] = [
    // NULL - must be present
    GdtEntry::NULL,
    // Kernel code
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_0)
            .union(GdtAccessFlags::SYSTEM)
            .union(GdtAccessFlags::EXECUTABLE)
            .union(GdtAccessFlags::PRIVILEGE),
        GdtEntryFlags::LONG_MODE,
    ),
    // Kernel data
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_0)
            .union(GdtAccessFlags::SYSTEM)
            .union(GdtAccessFlags::PRIVILEGE),
        GdtEntryFlags::LONG_MODE,
    ),
];

/// Loads early GDT
pub fn early_init() {
    unsafe {
        load(&mut GDT_BOOT);

        // Reload registers
        load_cs(SegmentSelector::new(GdtEntryType::KernelCode, Ring::Ring0));
        load_ds(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));
        load_es(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));
        load_ss(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));
    }
}
