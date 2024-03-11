use crate::arch::sync::Mutex;

use super::{idt::InterruptErrorStack, InterruptStack, IDT_ENTRIES};

static HANDLERS: Handlers = Handlers::const_new();

pub type InterruptHandler = fn(&mut InterruptStack);
pub type ExceptionHandler = fn(u64, &mut InterruptStack);

struct Handlers {
    db: Mutex<HandlerDb>,
}

struct HandlerDb {
    entries: [Option<Handler>; IDT_ENTRIES],
    counter: u8,
}

impl HandlerDb {
    fn register_with_index(&mut self, index: u8, handler: Handler) {
        self.entries[index as usize] = Some(handler);
    }

    fn register_with_autoincrement(&mut self, handler: Handler, force: bool) -> u8 {
        let index = self.counter;
        let entry = &mut self.entries[index as usize];
        if entry.is_some() && !force {
            panic!("attempt to override interrupt handler");
        }
        *entry = Some(handler);
        self.counter += 1;
        index
    }

    fn handler(&self, index: u8) -> &Option<Handler> {
        &self.entries[index as usize]
    }
}

impl Handlers {
    pub const fn const_new() -> Self {
        Self {
            db: Mutex::new(HandlerDb {
                entries: [None; IDT_ENTRIES],
                counter: 34,
            }),
        }
    }
}

impl Handlers {
    fn register_exception(&self, index: u8, handler: ExceptionHandler) {
        self.db
            .lock_disabling_interrupts()
            .register_with_index(index, Handler::Exception(handler));
    }

    fn register_interrupt(&self, handler: InterruptHandler) -> u8 {
        self.db
            .lock_disabling_interrupts()
            .register_with_autoincrement(Handler::Interrupt(handler), false)
    }

    fn handle(&self, index: u8, stack: &mut InterruptErrorStack) {
        let db = self.db.lock_disabling_interrupts();
        if let Some(handler) = db.handler(index) {
            handler.handle(stack);
        } else {
            log::error!("handler {index} not registered");
        }
    }
}

unsafe impl Send for Handlers {}
unsafe impl Sync for Handlers {}

#[derive(Clone, Copy)]
pub enum Handler {
    Exception(ExceptionHandler),
    Interrupt(InterruptHandler),
}

impl Handler {
    pub fn handle(&self, stack: &mut InterruptErrorStack) {
        match self {
            Handler::Exception(handler) => handler(stack.error_code, &mut stack.stack),
            Handler::Interrupt(handler) => handler(&mut stack.stack),
        }
    }
}

pub fn register_interrupt(handler: InterruptHandler) -> u8 {
    HANDLERS.register_interrupt(handler)
}

pub fn register_exception(index: u8, handler: ExceptionHandler) {
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
    HANDLERS.handle(isr as u8, stack);
}
