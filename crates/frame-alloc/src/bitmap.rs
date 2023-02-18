use core::ops::Range;

use ds::bitmap::Bitmap;
use easybit::align_up;

use crate::{FrameAlloc, PhysAddr};

const USED: bool = true;
const FREE: bool = false;

/// Simple O(n) frame allocator
pub struct BitmapAlloc<'a, const FRAME_SIZE: usize> {
    bitmap: Bitmap<'a>,
}

impl<'a, const FRAME_SIZE: usize> BitmapAlloc<'a, FRAME_SIZE> {
    pub fn build(storage: &'a mut [u8], free_ranges: impl Iterator<Item = Range<usize>>) -> Self {
        let mut bitmap = Bitmap::new(storage);
        bitmap.fill(USED);

        let mut this = Self { bitmap };

        free_ranges.for_each(|range| {
            let start = range.start;
            let end = range.end;

            let start = start / FRAME_SIZE;

            this.set_bitrange(start..end, FREE);
        });

        this
    }

    // TODO: consider implementing set_range on bitmap to avoid multiple div operations
    fn set_bitrange(&mut self, range: Range<usize>, v: bool) {
        range.for_each(|bit| {
            self.bitmap.set_bit(bit, v);
        })
    }
}

impl<'a, const FRAME_SIZE: usize, A: PhysAddr> FrameAlloc<FRAME_SIZE, A>
    for BitmapAlloc<'a, FRAME_SIZE>
{
    fn alloc(&mut self, size: usize) -> Option<A> {
        let size = align_up!(size, FRAME_SIZE) / FRAME_SIZE;

        // TODO: consider adding best_bet
        let bit = self.bitmap.find_first(None, size, FREE)?;

        self.set_bitrange(bit..bit + size, USED);

        Some(A::from(bit as u64))
    }

    fn free(&mut self, _phys_ptr: A) {}
}
