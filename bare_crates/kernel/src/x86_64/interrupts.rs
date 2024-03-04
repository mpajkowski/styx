mod handlers;
mod idt;
pub mod ioapic;
pub mod lapic;

pub const IDT_ENTRIES: usize = 256;

pub use idt::{init, init_ap, InterruptStack};

pub use handlers::{register_exception, register_interrupt};

use self::lapic::LOCAL_APIC;

pub fn notify_end_of_interrupt() {
    unsafe { LOCAL_APIC.get_unchecked() }
        .lock()
        .notify_end_of_interrupt()
}
