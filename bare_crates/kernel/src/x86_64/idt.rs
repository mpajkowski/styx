use core::{
    arch::asm,
    fmt::Debug,
    marker::PhantomData,
    mem::size_of,
    ops::{Index, IndexMut},
};

use crate::{
    addr::VirtAddr,
    arch::x86_64::{flags::Rflags, gdt},
    bits::BitManipulate,
};

use super::{
    gdt::{Ring, SegmentSelector},
    DescriptorPointer,
};

pub type HandlerFn = extern "x86-interrupt" fn(InterruptStackFrame);
pub type HandlerFnWithErrcode = extern "x86-interrupt" fn(InterruptStackFrame, errcode: u64);

pub type HandlerFnNever = extern "x86-interrupt" fn(InterruptStackFrame) -> !;
pub type HandlerFnNeverWithErrcode =
    extern "x86-interrupt" fn(InterruptStackFrame, errcode: u64) -> !;

#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_error: Entry<HandlerFn>,
    pub debug: Entry<HandlerFn>,
    pub non_maskable_interrupt: Entry<HandlerFn>,
    pub breakpoint: Entry<HandlerFn>,
    pub overflow: Entry<HandlerFn>,
    pub bound_range_exceeded: Entry<HandlerFn>,
    pub invalid_opcode: Entry<HandlerFn>,
    pub device_not_available: Entry<HandlerFn>,
    pub double_fault: Entry<HandlerFnNeverWithErrcode>,
    coprocessor_segment_overrun: Entry<HandlerFn>,
    pub invalid_tss: Entry<HandlerFnWithErrcode>,
    pub segment_not_present: Entry<HandlerFnWithErrcode>,
    pub stack_segment_fault: Entry<HandlerFnWithErrcode>,
    pub general_protection_fault: Entry<HandlerFnWithErrcode>,
    pub page_fault: Entry<HandlerFnWithErrcode>,
    _reserved_1: Entry<HandlerFn>,
    pub x87_floating_point: Entry<HandlerFn>,
    pub alignment_check: Entry<HandlerFnWithErrcode>,
    pub machine_check: Entry<HandlerFnNever>,
    pub simd_floating_point: Entry<HandlerFn>,
    pub virtualization: Entry<HandlerFn>,
    _reserved_2: [Entry<HandlerFn>; 9],
    pub security_exception: Entry<HandlerFnWithErrcode>,
    _reserved_3: Entry<HandlerFn>,
    pub interrupts: [Entry<HandlerFn>; 256 - 32],
}

impl<T: Into<usize>> Index<T> for InterruptDescriptorTable {
    type Output = Entry<HandlerFn>;

    fn index(&self, index: T) -> &Self::Output {
        let index = index.into();
        match index {
            0 => &self.divide_error,
            1 => &self.debug,
            2 => &self.non_maskable_interrupt,
            3 => &self.breakpoint,
            4 => &self.overflow,
            5 => &self.bound_range_exceeded,
            6 => &self.invalid_opcode,
            7 => &self.device_not_available,
            9 => &self.coprocessor_segment_overrun,
            16 => &self.x87_floating_point,
            19 => &self.simd_floating_point,
            20 => &self.virtualization,
            i @ 32..=255 => &self.interrupts[i - 32],
            i @ 15 | i @ 31 | i @ 21..=28 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is an diverging exception (must not return)", i),
            i => panic!("no entry with index {}", i),
        }
    }
}

impl<T: Into<usize>> IndexMut<T> for InterruptDescriptorTable {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let index = index.into();
        match index {
            0 => &mut self.divide_error,
            1 => &mut self.debug,
            2 => &mut self.non_maskable_interrupt,
            3 => &mut self.breakpoint,
            4 => &mut self.overflow,
            5 => &mut self.bound_range_exceeded,
            6 => &mut self.invalid_opcode,
            7 => &mut self.device_not_available,
            9 => &mut self.coprocessor_segment_overrun,
            16 => &mut self.x87_floating_point,
            19 => &mut self.simd_floating_point,
            20 => &mut self.virtualization,
            i @ 32..=255 => &mut self.interrupts[i - 32],
            i @ 15 | i @ 31 | i @ 21..=28 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is an diverging exception (must not return)", i),
            i => panic!("no entry with index {}", i),
        }
    }
}

impl Default for InterruptDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            divide_error: Entry::MINIMAL,
            debug: Entry::MINIMAL,
            non_maskable_interrupt: Entry::MINIMAL,
            breakpoint: Entry::MINIMAL,
            overflow: Entry::MINIMAL,
            bound_range_exceeded: Entry::MINIMAL,
            invalid_opcode: Entry::MINIMAL,
            device_not_available: Entry::MINIMAL,
            double_fault: Entry::MINIMAL,
            coprocessor_segment_overrun: Entry::MINIMAL,
            invalid_tss: Entry::MINIMAL,
            segment_not_present: Entry::MINIMAL,
            stack_segment_fault: Entry::MINIMAL,
            general_protection_fault: Entry::MINIMAL,
            page_fault: Entry::MINIMAL,
            _reserved_1: Entry::MINIMAL,
            x87_floating_point: Entry::MINIMAL,
            alignment_check: Entry::MINIMAL,
            machine_check: Entry::MINIMAL,
            simd_floating_point: Entry::MINIMAL,
            virtualization: Entry::MINIMAL,
            _reserved_2: [Entry::MINIMAL; 9],
            security_exception: Entry::MINIMAL,
            _reserved_3: Entry::MINIMAL,
            interrupts: [Entry::MINIMAL; 256 - 32],
        }
    }
    /// ## Safety
    pub unsafe fn load(&'static self) {
        let desc_ptr = DescriptorPointer {
            size: (size_of::<Self>() - 1) as u16,
            address: self as *const _ as u64,
        };

        asm!("lidt [{}]", in(reg) &desc_ptr, options(readonly, nostack, preserves_flags));
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Entry<F> {
    ptr_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    ptr_mid: u16,
    ptr_high: u32,
    reserved: u32,
    _handler_type: PhantomData<F>,
}

impl<F> Entry<F> {
    const MINIMAL: Self = Self {
        ptr_low: 0,
        gdt_selector: SegmentSelector::zeroed(),
        options: EntryOptions::MINIMAL,
        ptr_mid: 0,
        ptr_high: 0,
        reserved: 0,
        _handler_type: PhantomData,
    };

    pub fn set_handler_addr(&mut self, addr: VirtAddr) -> &mut EntryOptions {
        let addr = addr.as_u64();

        self.gdt_selector = gdt::cs::get();
        self.options.set_present(true);

        self.ptr_low = addr as u16;
        self.ptr_mid = (addr >> 16) as u16;
        self.ptr_high = (addr >> 32) as u32;

        debug_assert_eq!(self.handler_addr(), addr);

        &mut self.options
    }

    pub fn handler_addr(&self) -> u64 {
        self.ptr_low as u64 | ((self.ptr_mid as u64) << 16) | ((self.ptr_high as u64) << 32)
    }
}

macro_rules! impl_handler_fn {
    ($f: ident) => {
        impl Entry<$f> {
            #[inline]
            #[allow(clippy::fn_to_numeric_cast)]
            pub fn set_handler_fn(&mut self, handler: $f) -> &mut EntryOptions {
                let handler = VirtAddr::new(handler as u64);
                ::log::trace!("Handler: {:?}", handler);
                self.set_handler_addr(handler)
            }
        }
    };
}

impl_handler_fn!(HandlerFn);
impl_handler_fn!(HandlerFnWithErrcode);
impl_handler_fn!(HandlerFnNever);
impl_handler_fn!(HandlerFnNeverWithErrcode);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct EntryOptions(u16);

impl Default for EntryOptions {
    fn default() -> Self {
        Self::MINIMAL
    }
}

impl EntryOptions {
    const MINIMAL: Self = Self(0b1110_0000_0000);

    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    #[inline]
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    #[inline]
    pub fn set_privilege_level(&mut self, ring: Ring) -> &mut Self {
        self.0.set_range(13..15, ring as u16);
        self
    }

    #[inline]
    pub fn set_stack_index(&mut self, idx: u16) -> &mut Self {
        self.0.set_range(0..3, idx + 1);
        self
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub isp: VirtAddr,
    pub code_segment: SegmentSelector,
    pub cpu_flags: Rflags,
    pub stack_pointer: VirtAddr,
    pub stack_segment: SegmentSelector,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sizeof() {
        assert_eq!(size_of::<Entry<HandlerFn>>(), 16);
        assert_eq!(size_of::<InterruptDescriptorTable>(), 256 * 16);
    }
}
