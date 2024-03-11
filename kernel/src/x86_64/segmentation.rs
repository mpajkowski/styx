#![allow(unused)]

use core::{alloc::Layout, arch::asm, fmt::Debug, mem::size_of};

use raw_cpuid::ExtendedFeatures;

use crate::arch::{registers::Cr4, VirtAddr};

use super::{
    cpulocal::{self, CpuInfo, CpuLocal},
    heap,
};

#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct DescriptorPointer {
    pub size: u16,
    pub address: VirtAddr,
}

static mut READ_FS: unsafe fn() -> u64 = impl_msr::read_fs;
static mut READ_GS: unsafe fn() -> u64 = impl_msr::read_gs;
static mut WRITE_FS: unsafe fn(u64) = impl_msr::write_fs;
static mut WRITE_GS: unsafe fn(u64) = impl_msr::write_gs;

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
#[derive(Debug)]
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
unsafe fn load_gdt(table: &mut [GdtEntry]) {
    let ptr = DescriptorPointer {
        size: (core::mem::size_of_val(table) - 1) as u16,
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
                    other => panic!("unknown ring {other}, implementation or cpu bug"),
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

pub fn get_cs() -> SegmentSelector {
    let segment: u16;
    unsafe { asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags)) };

    SegmentSelector(segment)
}

#[inline(always)]
unsafe fn load_es(selector: SegmentSelector) {
    asm!("mov es, {0:x}", in(reg) selector.0, options(nomem, nostack))
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

const GDT_SIZE: usize = 10;
const GDT_TEMPLATE: [GdtEntry; GDT_SIZE] = [
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
    // Kernel Tls
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_0)
            .union(GdtAccessFlags::SYSTEM)
            .union(GdtAccessFlags::PRIVILEGE),
        GdtEntryFlags::LONG_MODE,
    ),
    // User data
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_3)
            .union(GdtAccessFlags::SYSTEM)
            .union(GdtAccessFlags::PRIVILEGE),
        GdtEntryFlags::LONG_MODE,
    ),
    // User code
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_3)
            .union(GdtAccessFlags::SYSTEM)
            .union(GdtAccessFlags::EXECUTABLE)
            .union(GdtAccessFlags::PRIVILEGE),
        GdtEntryFlags::LONG_MODE,
    ),
    // User data
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_3)
            .union(GdtAccessFlags::SYSTEM)
            .union(GdtAccessFlags::PRIVILEGE),
        GdtEntryFlags::LONG_MODE,
    ),
    // User Tls
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_3)
            .union(GdtAccessFlags::SYSTEM)
            .union(GdtAccessFlags::PRIVILEGE),
        GdtEntryFlags::LONG_MODE,
    ),
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            .union(GdtAccessFlags::RING_0)
            .union(GdtAccessFlags::TSS_AVAIL),
        GdtEntryFlags::NULL,
    ),
    GdtEntry::NULL,
];

/// Loads early GDT
pub fn early_init(feat: &ExtendedFeatures) {
    if feat.has_fsgsbase() {
        Cr4::read().union(Cr4::FSGSBASE).write();

        unsafe {
            READ_FS = impl_fsgsbase::read_fs;
            READ_GS = impl_fsgsbase::read_gs;
            WRITE_FS = impl_fsgsbase::write_fs;
            WRITE_GS = impl_fsgsbase::write_gs;
        }
    }

    unsafe {
        load_gdt(&mut GDT_BOOT);

        // Reload segment registers
        load_cs(SegmentSelector::new(GdtEntryType::KernelCode, Ring::Ring0));
        load_ds(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));
        load_es(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));
        load_ss(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));
    }

    // clear gs and fs
    write_gs(0);
    write_fs(0);
}

pub fn late_init(stack: VirtAddr, cpulocal: &mut CpuLocal) {
    let layout = Layout::for_value(&GDT_TEMPLATE);
    let mem = heap::alloc_from_layout(layout) as *mut GdtEntry;

    let gdt = unsafe { core::slice::from_raw_parts_mut(mem, GDT_SIZE) };

    gdt.copy_from_slice(&GDT_TEMPLATE);

    unsafe {
        let tss = &mut cpulocal.tss;
        let tss_ptr = tss as *mut Tss;

        let tss_low = GdtEntryType::TssLow as usize;
        let tss_hi = GdtEntryType::TssHi as usize;

        // write tss pointer
        gdt[tss_low].set_offset(tss_ptr as u32);
        gdt[tss_hi].set_raw((tss_ptr as u64) >> 32);

        // set tss limit
        gdt[tss_low].set_limit(size_of::<Tss>() as u32);

        tss.rsp[0] = stack;

        load_gdt(gdt);

        // Reload segment registers
        load_cs(SegmentSelector::new(GdtEntryType::KernelCode, Ring::Ring0));
        load_ds(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));
        load_es(SegmentSelector::new(GdtEntryType::KernelData, Ring::Ring0));

        load_tss(SegmentSelector::new(GdtEntryType::TssLow, Ring::Ring0));

        cpulocal.info.gdt = gdt;
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
        asm!("rdgsbase {}", out(reg) val);
        val
    }

    pub unsafe fn write_fs(val: u64) {
        asm!("wrfsbase {}", in(reg) val);
    }

    pub unsafe fn write_gs(val: u64) {
        asm!("wrgsbase {}", in(reg) val);
    }
}
