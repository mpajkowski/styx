use core::slice;

use easybit::align_up;
use frame_alloc::{
    bitmap::{BitmapAlloc, USED},
    FrameAlloc,
};
use limine_mini::memmap::EntryKind;

use crate::x86_64::limine::Limine;

use super::{sync::Mutex, PhysAddr};
pub const FRAME_SIZE: u64 = 0x1000;

static FRAME_ALLOC: Mutex<Option<PhysAlloc>> = Mutex::new(None);

pub struct PhysAlloc {
    bitmap_alloc: BitmapAlloc<'static, { FRAME_SIZE as usize }>,
}

impl PhysAlloc {
    pub fn with<T, F: FnMut(&mut Self) -> T>(mut fun: F) -> T {
        let mut lock = FRAME_ALLOC.lock_disabling_interrupts();
        let alloc = lock.as_mut().expect("Frame alloc not initialized yet");

        fun(alloc)
    }

    pub fn alloc_frame_size<A: From<u64>>(&mut self) -> Option<A> {
        self.bitmap_alloc.alloc(FRAME_SIZE as usize)
    }

    pub fn alloc<A: From<u64>>(&mut self, size: usize) -> Option<A> {
        self.bitmap_alloc.alloc(size)
    }
}

pub fn initialize(boot_info: &Limine) {
    let memory_map = &boot_info.memmap;

    for entry in memory_map.entries() {
        log::debug!("memory map entry: {entry:?}");
    }

    let usable_ranges = memory_map
        .entries()
        .filter(|e| e.kind == EntryKind::Usable && e.base != 0x1000);

    let max_addr = usable_ranges
        .clone()
        .map(|e| e.base + e.len)
        .max()
        .expect("Failed to find maximum usable addr, check your memory devices");

    let max_addr = align_up!(max_addr, FRAME_SIZE);

    let storage_len = max_addr / FRAME_SIZE / 8;

    log::debug!("Attempting to find entry of size {storage_len}b");

    let (storage, start) = usable_ranges
        .clone()
        .find_map(|e| {
            (e.len <= storage_len).then(|| {
                let start = PhysAddr::new_aligned::<FRAME_SIZE>(e.base).to_io().to_u64()
                    as *const u64 as *mut u64 as *mut u8;
                let storage = unsafe { slice::from_raw_parts_mut(start, storage_len as usize) };

                (storage, e.base as usize)
            })
        })
        .expect("Failed to find sufficient storage");

    let mut alloc = BitmapAlloc::build(
        storage,
        usable_ranges.map(|e| (e.base as usize)..(e.len as usize)),
    );

    alloc.mark_physical_region(start..storage_len as usize, USED);

    *FRAME_ALLOC.lock() = Some(PhysAlloc {
        bitmap_alloc: alloc,
    });
}
