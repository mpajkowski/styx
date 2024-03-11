use crate::arch::interrupts::ioapic;
use crate::x86_64::interrupts::{self, InterruptStack};

pub fn init() {
    let vec = interrupts::register_interrupt(pit_interrupt);
    ioapic::register_legacy_irq(0, vec, true);
}

fn pit_interrupt(_: &mut InterruptStack) {
    interrupts::notify_end_of_interrupt();
}
