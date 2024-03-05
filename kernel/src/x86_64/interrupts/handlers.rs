use core::cell::UnsafeCell;

use crate::arch::sync::Mutex;

use super::{idt::InterruptErrorStack, InterruptStack, IDT_ENTRIES};

static HANDLERS: Handlers = Handlers::const_new();

pub type InterruptHandler = fn(&mut InterruptStack);
pub type ExceptionHandler = fn(u64, &mut InterruptStack);

struct Handlers {
    entries: UnsafeCell<[Handler; IDT_ENTRIES]>,
    counter_lock: Mutex<u8>,
}

impl Handlers {
    pub const fn const_new() -> Self {
        Self {
            entries: UnsafeCell::new([Handler::None; IDT_ENTRIES]),
            counter_lock: Mutex::new(32),
        }
    }
}

impl Handlers {
    fn register_exception(&self, index: usize, handler: ExceptionHandler) {
        let _counter = self.counter_lock.lock_disabling_interrupts();

        // SAFETY: protected by counter_lock mutex guard
        let entries = unsafe { &mut *self.entries.get() };

        entries[index] = Handler::Exception(handler);
    }

    fn register_interrupt(&self, handler: InterruptHandler) -> u8 {
        let mut counter = self.counter_lock.lock_disabling_interrupts();

        let index = *counter;

        // SAFETY: protected by counter_lock mutex guard
        let entries = unsafe { &mut *self.entries.get() };
        entries[index as usize] = Handler::Interrupt(handler);

        *counter += 1;

        index
    }

    fn handler(&self, index: usize) -> &Handler {
        // SAFETY: interrupts won't fire if they're disabled (I hope)
        let entries = unsafe { &*self.entries.get() };
        &entries[index]
    }
}

unsafe impl Send for Handlers {}
unsafe impl Sync for Handlers {}

#[derive(Clone, Copy)]
pub enum Handler {
    Exception(ExceptionHandler),
    Interrupt(InterruptHandler),
    None,
}

impl Handler {
    pub fn handle(&self, stack: &mut InterruptErrorStack) {
        match self {
            Handler::Exception(handler) => handler(stack.error_code, &mut stack.stack),
            Handler::Interrupt(handler) => handler(&mut stack.stack),
            Handler::None => log::error!("Handler not registered"),
        }
    }
}

pub fn register_interrupt(handler: InterruptHandler) -> u8 {
    HANDLERS.register_interrupt(handler)
}

pub fn register_exception(index: usize, handler: ExceptionHandler) {
    HANDLERS.register_exception(index, handler);
}

macro_rules! make_exception {
    ($name:ident => $message:expr) => {
        pub fn $name(error: u64, stack: &mut InterruptStack) {
            log::error!("Exception: {}, error: {}", $message, error);
            log::error!("Stack: {:#?}", stack);
        }
    };
}

make_exception!(divide_by_zero => "Division by zero");
make_exception!(debug => "Debug");
make_exception!(non_maskable => "Non Maskable");
make_exception!(breakpoint => "Breakpoint");
make_exception!(overflow => "Stack Overflow");
make_exception!(bound_range => "Out of Bounds");
make_exception!(invalid_opcode => "Invalid opcode");
make_exception!(device_not_available => "Device not available");

pub fn double_fault(error: u64, stack: &mut InterruptStack) {
    log::error!("Exception: DOUBLE FAULT, error: {error}");
    log::error!("Stack: {stack:#?}");

    loop {
        crate::arch::sync::hlt();
    }
}

make_exception!(invalid_tss => "Invalid tss");
make_exception!(segment_not_present => "Segment not Present");
make_exception!(stack_segment => "Stack Segment Fault");
make_exception!(protection => "Protection Fault");
make_exception!(page_fault => "Page Fault");
make_exception!(fpu_fault => "FPU floating point fault");
make_exception!(alignment_check => "Alignment check fault");
make_exception!(machine_check => "Machine check fault");
make_exception!(simd => "SIMD floating point fault");
make_exception!(virtualization => "Virtualization fault");
make_exception!(security => "Security exception");

pub fn handle(isr: u64, stack: &mut InterruptErrorStack) {
    let handler = HANDLERS.handler(isr as usize);
    handler.handle(stack);
}
