//! Physical frame structs

use super::page::{AddressNotAligned, PageSize};
use crate::arch::PhysAddr;

/// Error that may occur during allocating/accessing page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// Frame of size bigger than `FRAME_SIZE`
    HugeFrame,
    /// PageTableEntry not marked as present
    NotPresent,
}

/// Physical memory frame
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    start_addr: PhysAddr,
}

impl Frame {
    /// Creates frame from `addr` and `size`
    pub fn from_start_address(addr: PhysAddr, size: PageSize) -> Result<Self, AddressNotAligned> {
        if addr.is_aligned(size as u64) {
            Ok(Self { start_addr: addr })
        } else {
            Err(AddressNotAligned)
        }
    }

    /// Create new Frame
    ///
    /// SAFETY: `addr` must be aligned to desired `PageSize`
    pub fn new_unchecked(addr: PhysAddr) -> Self {
        Self { start_addr: addr }
    }

    /// Creates frame from `addr` and `size`
    ///
    /// Aligns down `addr` to `size` if needed
    pub fn containing_addr(addr: PhysAddr, size: PageSize) -> Self {
        Self {
            start_addr: addr.align_down(size as u64),
        }
    }

    /// Returns physical start address of this `Frame`
    pub fn start_addr(&self) -> PhysAddr {
        self.start_addr
    }
}

impl From<u64> for Frame {
    fn from(value: u64) -> Self {
        Self::new_unchecked(PhysAddr::new_unchecked(value))
    }
}
