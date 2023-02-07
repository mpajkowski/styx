use crate::arch::sync::Mutex;

use super::{idt::InterruptErrorStack, InterruptStack, IDT_ENTRIES};

static mut HANDLERS: Handlers = Handlers::const_new();

pub type InterruptHandler = fn(&mut InterruptStack);
pub type ExceptionHandler = fn(u64, &mut InterruptStack);

pub struct Handlers([Handler; IDT_ENTRIES]);

impl Handlers {
    pub const fn const_new() -> Self {
        Self([Handler::None; IDT_ENTRIES])
    }
}

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

pub fn register_handler(handler: InterruptHandler) -> usize {
    static COUNTER: Mutex<usize> = Mutex::new(32);

    let mut counter = COUNTER.lock_disabling_interrupts();
    unsafe { HANDLERS.0[*counter] = Handler::Interrupt(handler) };

    let ret = *counter;
    *counter += 1;

    ret
}

pub unsafe fn register_exception(index: usize, handler: ExceptionHandler) {
    HANDLERS.0[index] = Handler::Exception(handler);
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
    log::error!("Exception: PAGE_FAULT, error: {error}");
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
    let handler = unsafe { HANDLERS.0[isr as usize] };
    handler.handle(stack);
}
