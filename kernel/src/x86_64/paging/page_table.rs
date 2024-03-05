//! Page tables and entries

use core::ops::{Index, IndexMut};

use bitflags::bitflags;

use crate::arch::{PhysAddr, FRAME_SIZE};

use super::{
    frame::{Frame, FrameError},
    page::PageSize,
};

/// Page Table (aligned to MEM_FRAME_SIZE)
#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Creates new `PageTable`
    pub const fn new() -> Self {
        const UNUSED: PageTableEntry = PageTableEntry::new();

        Self {
            entries: [UNUSED; 512],
        }
    }

    /// Returns iterator of entries
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        self.entries.iter()
    }

    /// Returns mutable iterator of entries
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        self.entries.iter_mut()
    }

    /// Marks all entries as unused
    pub fn zero(&mut self) {
        self.iter_mut().for_each(|entry| entry.set_unused());
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 512, "Page table index >= 512");
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < 512, "Page table index >= 512");
        &mut self.entries[index]
    }
}

/// Stores per-page properties
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Creates unused `PageTableEntry`
    pub const fn new() -> Self {
        Self(0)
    }

    /// Returns whether this entry is unused
    #[inline]
    pub const fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Marks entry as unused
    #[inline]
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Returns flags
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }

    /// Sets given flags
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.0 = self.addr().to_u64() | flags.bits();
    }

    /// Returns physical address
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new_unchecked(self.0 & 0x000f_ffff_ffff_f000)
    }

    /// Sets physical address
    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        debug_assert!(addr.is_aligned(FRAME_SIZE));
        self.0 = addr.to_u64() | flags.bits();
    }

    /// Links with frame
    pub fn set_frame(&mut self, frame: Frame, flags: PageTableFlags) {
        debug_assert!(!flags.contains(PageTableFlags::HUGE_PAGE));
        self.set_addr(frame.start_addr(), flags)
    }

    /// Returns linked frame
    pub fn frame(&self) -> Result<Frame, FrameError> {
        let flags = self.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            Err(FrameError::NotPresent)
        } else if flags.contains(PageTableFlags::HUGE_PAGE) {
            Err(FrameError::HugeFrame)
        } else {
            Ok(Frame::containing_addr(self.addr(), PageSize::Normal4K))
        }
    }
}

bitflags! {
    /// Page table flags
    #[derive(Clone, Copy, Debug)]
    pub struct PageTableFlags: u64 {
        /// Page table is marked as present
        const PRESENT = 1;
        /// Page table is writable
        const WRITABLE = 1 << 1;
        /// Page table is accessible from CPL3
        const USER_ACCESSIBLE = 1 << 2;
        /// Per page write though setting
        const WRITE_THROUGH = 1 << 3;
        /// Disables cache
        const NO_CACHE = 1<< 4;
        /// Was this page accessed? Set by MMU
        const ACCCESSED = 1 << 5;
        /// Was this page written? Set by MMU
        const DIRTY = 1 << 6;
        /// Huge Page bit
        const HUGE_PAGE = 1 << 7;
        /// Global Page Bit
        const GLOBAL = 1 << 8;
        /// NO_EXECUTE flag - must be enabled in Efer
        const NO_EXECUTE = 1 << 63;
    }
}
