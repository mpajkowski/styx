use core::{arch::asm, mem};

use bitflags::bitflags;
use easybit::BitManipulate;

use crate::x86_64::DescriptorPointer;

use super::{
    gdt::{self, Ring, SegmentSelector},
    registers::{IretRegisters, PreservedRegisters, ScratchRegisters},
    VirtAddr,
};

const IDT_ENTRIES: usize = 256;

#[repr(align(0x10))]
struct InterruptDescriptorTable([Entry; IDT_ENTRIES]);

impl InterruptDescriptorTable {
    pub const fn const_new() -> Self {
        Self([Entry::NULL; IDT_ENTRIES])
    }
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::const_new();

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Entry {
    ptr_low: u16,
    gdt_selector: SegmentSelector,
    ist: u8,
    flags: Flags,
    ptr_mid: u16,
    ptr_high: u32,
    _reserved: u32,
}

impl Entry {
    const NULL: Self = Self {
        ptr_low: 0,
        gdt_selector: SegmentSelector::zeroed(),
        ist: 0,
        flags: Flags::zeroed(),
        ptr_mid: 0,
        ptr_high: 0,
        _reserved: 0,
    };

    fn set_handler_addr(&mut self, addr: VirtAddr) {
        let addr = addr.as_u64();

        self.gdt_selector = unsafe { gdt::get_cs() };
        self.flags = Flags::PRESENT | Flags::RING_0 | Flags::INTERRUPT;

        self.ptr_low = addr as u16;
        self.ptr_mid = (addr >> 16) as u16;
        self.ptr_high = (addr >> 32) as u32;
    }
}

bitflags! {
    struct Flags: u8 {
        const PRESENT = 1 << 7;
        const RING_0 = 0 << 5;
        const RING_1 = 1 << 5;
        const RING_2 = 2 << 5;
        const RING_3 = 3 << 5;
        const SS = 1 << 4;
        const INTERRUPT = 0xe;
        const TRAP = 0xf;
    }
}

impl Flags {
    pub const fn zeroed() -> Self {
        Flags { bits: 0 }
    }
}

pub struct InterruptStack {
    pub preserved: PreservedRegisters,
    pub scratch: ScratchRegisters,
    pub iret: IretRegisters,
}

#[no_mangle]
pub extern "C" fn generic_irq_handler() {
    log::info!("Called generic_interrupt_handler");
}

#[inline(always)]
unsafe fn load() {
    let ptr = DescriptorPointer {
        size: (core::mem::size_of::<InterruptDescriptorTable>() - 1) as u16,
        address: &IDT as *const _ as u64,
    };

    asm!("lidt [{}]", in(reg) &ptr, options(nostack));
}

pub fn init() {
    extern "C" {
        static irq_handler_table: [VirtAddr; IDT_ENTRIES];
    }

    unsafe {
        // assign handlers
        irq_handler_table
            .iter()
            .enumerate()
            .filter(|(_, addr)| !addr.is_zero())
            .for_each(|(idx, addr)| {
                IDT.0[idx].set_handler_addr(*addr);
            });

        load();
    }
}
