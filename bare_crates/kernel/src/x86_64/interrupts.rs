pub mod handlers;
mod idt;

pub const IDT_ENTRIES: usize = 256;

pub use idt::{init, InterruptStack};
