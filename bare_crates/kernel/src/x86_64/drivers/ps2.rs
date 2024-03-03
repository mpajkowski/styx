use crate::x86_64::interrupts::{self, InterruptStack};
use crate::x86_64::ioport;

const PS2_IOPORT: u16 = 0x60;

pub fn init() {
    let vec = interrupts::register_interrupt(ps2_kbd_interrupt);
}

fn ps2_kbd_interrupt(_: &mut InterruptStack) {
    let keycode = unsafe { ioport::read_u8(PS2_IOPORT) };

    log::info!("kbd code: {keycode:x}");
}
