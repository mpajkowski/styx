mod handlers;
mod idt;
pub mod lapic;
pub mod legacy_pic;
pub mod ioapic;

pub const IDT_ENTRIES: usize = 256;

use core::{
    mem,
    sync::atomic::{AtomicU8, Ordering},
};

pub use idt::{init, InterruptStack};

pub use handlers::{register_exception, register_interrupt};

use self::{lapic::LOCAL_APIC, legacy_pic::PIC8529PAIR};

static INTERRUPT_COMPLETER: InterruptSystemWrapper = InterruptSystemWrapper::legacy_pics();

pub fn notify_end_of_interrupt(interrupt: u8) {
    INTERRUPT_COMPLETER.notify_end_of_interrupt(interrupt);
}

pub fn switch_to_apic() {
    PIC8529PAIR.lock().disable();
    INTERRUPT_COMPLETER.set_system(InterruptSystem::Apic);
}

struct InterruptSystemWrapper(AtomicU8);

impl InterruptSystemWrapper {
    pub const fn legacy_pics() -> Self {
        Self(AtomicU8::new(InterruptSystem::Pics as u8))
    }
}

impl InterruptSystemWrapper {
    fn notify_end_of_interrupt(&self, interrupt: u8) {
        let system: InterruptSystem = unsafe { mem::transmute(self.0.load(Ordering::SeqCst)) };
        system.notify_end_of_interrupt(interrupt);
    }

    fn set_system(&self, system: InterruptSystem) {
        self.0.store(system as u8, Ordering::SeqCst);
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum InterruptSystem {
    Pics = 0,
    Apic = 1,
}

impl InterruptSystem {
    fn notify_end_of_interrupt(self, interrupt: u8) {
        match self {
            InterruptSystem::Pics => PIC8529PAIR.lock().notify_end_of_interrupt(interrupt),
            InterruptSystem::Apic => unsafe { LOCAL_APIC.get_unchecked() }
                .lock()
                .notify_end_of_interrupt(),
        }
    }
}
