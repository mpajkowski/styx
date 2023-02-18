//! Utilities for mapping and translating

use crate::arch::{PhysAddr, PhysAlloc, VirtAddr};

use super::{
    frame::{Frame, FrameError},
    page::{AddressNotAligned, MapperFlush, Page, PageSize},
    page_table::{PageTable, PageTableEntry, PageTableFlags},
};

/// Page table walker and mapper
pub struct OffsetTable<'a> {
    level_4_table: &'a mut PageTable,
    walker: PageTableWalker,
}

impl<'a> OffsetTable<'a> {
    /// Creates new `OffsetTable`
    ///
    /// # Safety
    /// User must provide valid memory offset
    pub unsafe fn new(level_4_table: &'a mut PageTable, offset: VirtAddr) -> Self {
        Self {
            level_4_table,
            walker: PageTableWalker::new(offset),
        }
    }

    /// Translates given virtual address
    #[allow(clippy::inconsistent_digit_grouping)]
    pub fn translate(&self, addr: VirtAddr) -> TranslateResult {
        let p4 = &self.level_4_table;

        let p3 = match self.walker.next_table(&p4[addr.p4_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                panic!("level 4 huge page :///");
            }
        };

        let p2 = match self.walker.next_table(&p3[addr.p3_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p3[addr.p3_index()];
                let frame = Frame::containing_addr(entry.addr(), PageSize::Huge1G);
                let offset = addr.to_u64() & 0o7_777_777_777;
                let flags = entry.flags();
                return TranslateResult::Mapped {
                    frame,
                    offset,
                    flags,
                    size: PageSize::Huge1G,
                };
            }
        };

        let p1 = match self.walker.next_table(&p2[addr.p2_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p2[addr.p2_index()];
                let frame = Frame::containing_addr(entry.addr(), PageSize::Huge2M);
                let offset = addr.to_u64() & 0o777_7777;
                let flags = entry.flags();
                return TranslateResult::Mapped {
                    frame,
                    offset,
                    flags,
                    size: PageSize::Huge2M,
                };
            }
        };

        let entry = &p1[addr.p1_index()];

        if entry.is_unused() {
            return TranslateResult::NotMapped;
        }

        let frame = match Frame::from_start_address(entry.addr(), PageSize::Normal4K) {
            Ok(frame) => frame,
            Err(AddressNotAligned) => return TranslateResult::InvalidFrameAddress(entry.addr()),
        };

        let offset = u64::from(addr.offset());
        let flags = entry.flags();

        TranslateResult::Mapped {
            frame,
            offset,
            flags,
            size: PageSize::Normal4K,
        }
    }

    /// Maps `Page` to `Frame` and creates translation entries
    pub fn map(
        &mut self,
        size: PageSize,
        page: Page,
        frame: Frame,
        flags: PageTableFlags,
        alloc: &mut PhysAlloc,
    ) -> Result<MapperFlush, MapToError> {
        //log::trace!("mapping {page:?} to {frame:?}, flags: {flags:?}, size: {size:?}");
        let parent_table_flags = flags
            & (PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE);

        match size {
            PageSize::Normal4K => self.map_4_kib(page, frame, parent_table_flags, flags, alloc),
            PageSize::Huge2M => self.map_2_mib(page, frame, parent_table_flags, flags, alloc),
            PageSize::Huge1G => self.map_1_gib(page, frame, parent_table_flags, flags, alloc),
        }
    }

    fn map_1_gib(
        &mut self,
        page: Page,
        frame: Frame,
        parent_table_flags: PageTableFlags,
        flags: PageTableFlags,
        alloc: &mut PhysAlloc,
    ) -> Result<MapperFlush, MapToError> {
        let p4 = &mut self.level_4_table;
        let p3 =
            self.walker
                .create_next_table(&mut p4[page.p4_index()], parent_table_flags, alloc)?;

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }

        p3[page.p3_index()].set_addr(frame.start_addr(), flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn map_2_mib(
        &mut self,
        page: Page,
        frame: Frame,
        parent_table_flags: PageTableFlags,
        flags: PageTableFlags,
        alloc: &mut PhysAlloc,
    ) -> Result<MapperFlush, MapToError> {
        //let t = self.translate(page.start_addr());
        //log::info!("translate: {t:?}");
        let p4 = &mut self.level_4_table;
        let p3 =
            self.walker
                .create_next_table(&mut p4[page.p4_index()], parent_table_flags, alloc)?;
        let p2 =
            self.walker
                .create_next_table(&mut p3[page.p3_index()], parent_table_flags, alloc)?;

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }

        p2[page.p2_index()].set_addr(frame.start_addr(), flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn map_4_kib(
        &mut self,
        page: Page,
        frame: Frame,
        parent_table_flags: PageTableFlags,
        flags: PageTableFlags,
        alloc: &mut PhysAlloc,
    ) -> Result<MapperFlush, MapToError> {
        let p4 = &mut self.level_4_table;
        let p3 =
            self.walker
                .create_next_table(&mut p4[page.p4_index()], parent_table_flags, alloc)?;
        let p2 =
            self.walker
                .create_next_table(&mut p3[page.p3_index()], parent_table_flags, alloc)?;
        let p1 =
            self.walker
                .create_next_table(&mut p2[page.p2_index()], parent_table_flags, alloc)?;

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }

        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }
}

#[derive(Debug, Clone, Copy)]
struct PageTableWalker {
    offset: VirtAddr,
}

impl PageTableWalker {
    fn new(offset: VirtAddr) -> Self {
        Self { offset }
    }

    #[inline]
    fn next_table<'a>(
        &self,
        entry: &'a PageTableEntry,
    ) -> Result<&'a PageTable, PageTableWalkError> {
        let addr = self.offset.to_u64() + entry.frame()?.start_addr().to_u64();
        let table_ptr = addr as *const PageTable;

        Ok(unsafe { &*table_ptr })
    }

    #[inline]
    fn next_table_mut<'a>(
        &self,
        entry: &'a mut PageTableEntry,
    ) -> Result<&'a mut PageTable, PageTableWalkError> {
        let addr = self.offset.to_u64() + entry.frame()?.start_addr().to_u64();
        let table_ptr = addr as *mut PageTable;

        Ok(unsafe { &mut *table_ptr })
    }

    #[inline]
    fn create_next_table<'a>(
        &self,
        entry: &'a mut PageTableEntry,
        flags: PageTableFlags,
        alloc: &mut PhysAlloc,
    ) -> Result<&'a mut PageTable, PageTableCreateError> {
        let mut created = false;

        if entry.is_unused() {
            let frame: Frame = match alloc.alloc_frame_size() {
                Some(f) => f,
                None => return Err(PageTableCreateError::FrameAllocationFailed),
            };

            entry.set_frame(frame, flags);
            created = true;
        } else if !flags.is_empty() && !entry.flags().contains(flags) {
            entry.set_flags(entry.flags() | flags);
        }

        let page_table = match self.next_table_mut(entry) {
            Err(PageTableWalkError::MappedToHugePage) => {
                return Err(PageTableCreateError::MappedToHugePage)
            }
            Err(PageTableWalkError::NotMapped) => panic!("Not mapped frame at this point"),
            Ok(pt) => pt,
        };

        if created {
            page_table.zero();
        }

        Ok(page_table)
    }
}

/// Error occurred during traversing page structures
#[derive(Debug)]
pub enum PageTableWalkError {
    /// Page is not mapped
    NotMapped,
    /// Page mapped to huge page
    MappedToHugePage,
}

impl From<FrameError> for PageTableWalkError {
    fn from(fe: FrameError) -> Self {
        match fe {
            FrameError::HugeFrame => Self::MappedToHugePage,
            FrameError::NotPresent => Self::NotMapped,
        }
    }
}

/// Error occurred during crating new page
#[derive(Debug)]
pub enum PageTableCreateError {
    /// Called on huge page
    MappedToHugePage,
    /// Physical allocator error
    FrameAllocationFailed,
}

/// Mapping error
#[derive(Debug)]
pub enum MapToError {
    /// Attempt to map present frame
    PageAlreadyMapped(Frame),
    /// Attempt to map to huge page
    PageEntryHugePage,
    /// Physical allocator error
    FrameAllocationFailed,
}

impl From<PageTableCreateError> for MapToError {
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => Self::PageEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => Self::FrameAllocationFailed,
        }
    }
}

/// Translation result
#[derive(Debug)]
pub enum TranslateResult {
    /// Mapped frame - mapping properties
    Mapped {
        /// page size
        size: PageSize,
        /// physical frame
        frame: Frame,
        /// addr offset
        offset: u64,
        /// entry flags
        flags: PageTableFlags,
    },
    /// Page is not mapped
    NotMapped,

    /// Page is mapped to invalid physical address
    InvalidFrameAddress(PhysAddr),
}
