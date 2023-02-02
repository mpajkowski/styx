mod drivers;
mod entrypoint;
mod gdt;
mod ioport;
mod limine;
mod logger;

mod addr;
pub mod sync;

pub use addr::{PhysAddr, VirtAddr, VirtAddrInvalid};

#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
struct DescriptorPointer {
    pub size: u16,
    pub address: u64,
}
