use core::arch::asm;

use bitflags::bitflags;

use crate::{
    arch::{
        interrupts::handlers::register_exception,
        registers::{IretRegisters, PreservedRegisters, ScratchRegisters},
        VirtAddr,
    },
    x86_64::{
        gdt::{self, SegmentSelector},
        DescriptorPointer,
    },
};

use super::{handlers, IDT_ENTRIES};

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

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InterruptErrorStack {
    pub error_code: u64,
    pub stack: InterruptStack,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InterruptStack {
    pub preserved: PreservedRegisters,
    pub scratch: ScratchRegisters,
    pub iret: IretRegisters,
}

#[no_mangle]
pub extern "C" fn generic_irq_handler(isr: u64, stack: *mut InterruptErrorStack) {
    log::debug!("Exception: {isr}");

    let stack = unsafe { &mut *stack };

    handlers::handle(isr, stack);
}

#[inline(always)]
unsafe fn load() {
    let ptr = DescriptorPointer {
        size: (core::mem::size_of::<InterruptDescriptorTable>() - 1) as u16,
        address: VirtAddr::new_unchecked(&IDT as *const _ as u64),
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

        register_exception(0, super::handlers::divide_by_zero);
        register_exception(1, super::handlers::debug);
        register_exception(2, super::handlers::non_maskable);
        register_exception(3, super::handlers::breakpoint);
        register_exception(4, super::handlers::overflow);
        register_exception(5, super::handlers::bound_range);
        register_exception(6, super::handlers::invalid_opcode);
        register_exception(7, super::handlers::device_not_available);
        register_exception(8, super::handlers::double_fault);
        register_exception(10, super::handlers::invalid_tss);
        register_exception(11, super::handlers::segment_not_present);
        register_exception(12, super::handlers::stack_segment);
        register_exception(13, super::handlers::protection);
        register_exception(14, super::handlers::page_fault);
        register_exception(16, super::handlers::fpu_fault);
        register_exception(17, super::handlers::alignment_check);
        register_exception(18, super::handlers::machine_check);
        register_exception(19, super::handlers::simd);
        register_exception(20, super::handlers::virtualization);
        register_exception(30, super::handlers::security);

        load();
    }
}
