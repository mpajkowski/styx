use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

use linked_list_allocator::Heap;

use crate::arch::FRAME_SIZE;
use crate::x86_64::sync::Mutex;
use crate::x86_64::{PhysAlloc, VirtAddr};

use super::paging::{
    address_space::AddressSpace,
    frame::Frame,
    page::{Page, PageSize},
    page_table::PageTableFlags,
};

pub const HEAP_SIZE: usize = PageSize::Huge2M as u64 as usize;
pub const HEAP_START: VirtAddr = VirtAddr::new_unchecked(0xffff_f800_0000_0000);

pub fn initialize() {
    let heap = HeapAllocator::init_heap();
    *crate::HEAP_ALLOC.inner.lock() = heap;
}

pub struct HeapAllocator {
    inner: Mutex<Heap>,
}

impl HeapAllocator {
    pub const fn uninitialized() -> Self {
        Self {
            inner: Mutex::new(Heap::empty()),
        }
    }

    fn init_heap() -> Heap {
        let mut address_space = AddressSpace::active();
        let mut offset_table = address_space.offset_table();

        PhysAlloc::with(|phys_alloc| {
            let max = HEAP_START + HEAP_SIZE as u64;
            let mut addr = HEAP_START;

            while addr < max {
                let frame: Frame = phys_alloc
                    .alloc_frame_size()
                    .expect("Failed to allocate physical frame for heap");

                let page = Page::containing_addr(addr, PageSize::Normal4K);

                offset_table
                    .map(
                        PageSize::Normal4K,
                        page,
                        frame,
                        PageTableFlags::WRITABLE | PageTableFlags::PRESENT,
                        phys_alloc,
                    )
                    .expect("Failed to map heap frame")
                    .flush();

                addr = addr + FRAME_SIZE;
            }
        });

        unsafe { Heap::new(HEAP_START.as_mut_ptr(), HEAP_SIZE) }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner
            .lock()
            .allocate_first_fit(layout)
            .map(|ptr| ptr.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).expect("passed null pointer");
        self.inner.lock().deallocate(ptr, layout)
    }
}
