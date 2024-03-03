mod drivers;
mod entrypoint;
mod gdt;
mod heap;
mod ioport;
mod limine;
mod logger;
mod msr;

pub mod acpi;
pub mod addr;
pub mod ap;
pub mod cpulocal;
pub mod features;
pub mod fsgs;
pub mod interrupts;
pub mod paging;
pub mod pmm;
pub mod registers;
pub mod sync;

pub use addr::{PhysAddr, VirtAddr, VirtAddrInvalid};
pub use heap::HeapAllocator;
pub use pmm::*;

#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
struct DescriptorPointer {
    pub size: u16,
    pub address: VirtAddr,
}
