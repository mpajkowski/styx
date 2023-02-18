//! Virtual pages structs

use core::arch::asm;

use crate::arch::{VirtAddr, FRAME_SIZE};

/// Page
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Page {
    start_addr: VirtAddr,
}

/// Given address is not page-size aligned
#[derive(Debug)]
pub struct AddressNotAligned;

/// Size of page
#[repr(u64)]
#[derive(Debug)]
pub enum PageSize {
    /// Regular page size
    Normal4K = FRAME_SIZE,
    /// Huge Page 2M
    Huge2M = Self::Normal4K as u64 * 512,
    /// Huge Page 1G
    Huge1G = Self::Huge2M as u64 * 512,
}

impl Page {
    /// Creates new `Page` from `start_addr` and `size`
    pub const fn new(start_addr: VirtAddr, size: PageSize) -> Result<Self, AddressNotAligned> {
        if start_addr.is_aligned(size as u64) {
            Ok(Page { start_addr })
        } else {
            Err(AddressNotAligned)
        }
    }

    /// Creates new `Page` from `start_addr` and `size`
    ///
    /// Aligns down `start_addr` to `size` if needed
    pub const fn containing_addr(start_addr: VirtAddr, size: PageSize) -> Self {
        Self {
            start_addr: start_addr.align_down(size as u64),
        }
    }

    /// Returns start addr of page
    pub const fn start_addr(self) -> VirtAddr {
        self.start_addr
    }

    /// Returns P1 table index (aka Table)
    pub const fn p1_index(self) -> usize {
        self.start_addr.p1_index()
    }

    /// Returns P2 table index (aka Directory)
    pub const fn p2_index(self) -> usize {
        self.start_addr.p2_index()
    }

    /// Returns P3 table index (Directory Pointer)
    pub const fn p3_index(self) -> usize {
        self.start_addr.p3_index()
    }

    /// Returns P4 table index (Directory Pointer)
    pub const fn p4_index(self) -> usize {
        self.start_addr.p4_index()
    }

    pub const fn to_u64(self) -> u64 {
        self.start_addr.to_u64()
    }
}

/// Page address change
///
/// Must be either flushed or ignored
#[derive(Debug)]
#[must_use = "Page Table changes must be flushed or ignored"]
pub struct MapperFlush(Page);

impl MapperFlush {
    /// Creates new flush promise
    pub fn new(page: Page) -> Self {
        Self(page)
    }

    /// Flushes using `invlpg`
    pub fn flush(self) {
        unsafe {
            asm!("invlpg [{}]", in(reg) self.0.to_u64(), options(nostack, preserves_flags));
        }
    }

    /// Discards flush promise
    pub fn ignore(self) {}
}
