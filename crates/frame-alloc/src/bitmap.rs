use core::ops::Range;

use ds::bitmap::Bitmap;
use easybit::align_up;

use crate::{FrameAlloc, PhysAddr};

pub const USED: bool = true;
pub const FREE: bool = false;

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
            this.mark_physical_region(range, FREE);
        });

        this
    }

    // TODO: consider implementing set_range on bitmap to avoid multiple div operations
    pub fn mark_physical_region(&mut self, region: Range<usize>, v: bool) {
        let start = region.start / FRAME_SIZE;
        let end = region.end / FRAME_SIZE;

        self.set_bitrange(start..end, v);
    }

    fn set_bitrange(&mut self, region_scaled: Range<usize>, v: bool) {
        region_scaled.for_each(|bit| {
            self.bitmap.set_bit(bit, v);
        });
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

        Some(A::from((bit * FRAME_SIZE) as u64))
    }

    fn free(&mut self, _phys_ptr: A) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use easybit::align_down;
    use rstest::rstest;

    #[test]
    fn build() {
        const MAX: usize = 0x100000;
        let usable_ranges = [(0x1000..0x4000), (0x8000..MAX)];
        let bitmap = &mut [0_u8; MAX / 0x1000 / 8];
        let alloc = BitmapAlloc::<0x1000>::build(bitmap, usable_ranges.into_iter());

        assert_addr(&alloc, 0x0000, USED);
        assert_addr(&alloc, 0x1000, FREE);
        assert_addr(&alloc, 0x2000, FREE);
        assert_addr(&alloc, 0x3000, FREE);
        assert_addr(&alloc, 0x4000, USED);
        assert_addr(&alloc, 0x5000, USED);
        assert_addr(&alloc, 0x6000, USED);
        assert_addr(&alloc, 0x7000, USED);
        assert_addr(&alloc, 0x8000, FREE);
        assert_addr(&alloc, 0x9000, FREE);
        assert_addr(&alloc, 0x10000, FREE);
    }

    #[rstest]
    #[case(0..0x10000, USED)]
    fn mark(#[case] range: Range<usize>, #[case] value: bool) {
        const MAX: usize = 0x100000;
        let usable_ranges = [(0x1000..0x3000), (0x8000..MAX)];
        let bitmap = &mut [0_u8; MAX / 0x1000 / 8];
        let mut alloc = BitmapAlloc::<0x1000>::build(bitmap, usable_ranges.into_iter());

        alloc.mark_physical_region(range, value);
    }

    #[rstest]
    #[case::alloc_0x1000(0x1000, Some(0x1000))]
    #[case::alloc_0x2000(0x2000, Some(0x1000))]
    #[case::alloc_0x3000_move_to_next_range(0x3000, Some(0x8000))]
    fn alloc(#[case] size: usize, #[case] expected_pointer: Option<u64>) {
        const MAX: usize = 0x100000;
        let usable_ranges = [(0x1000..0x3000), (0x8000..MAX)];
        let bitmap = &mut [0_u8; MAX / 0x1000 / 8];
        let mut alloc = BitmapAlloc::<0x1000>::build(bitmap, usable_ranges.into_iter());

        let pointer: Option<u64> = alloc.alloc(size);

        assert_eq!(pointer, expected_pointer);

        if let Some(pointer) = pointer {
            assert_addr(&alloc, pointer as usize, USED);
        }
    }

    #[track_caller]
    fn assert_addr(alloc: &BitmapAlloc<0x1000>, addr: usize, value: bool) {
        let bit = align_down!(addr, 0x1000) / 0x1000;

        if value == USED {
            assert_eq!(
                alloc.bitmap.read_bit(bit),
                Some(USED),
                "Addr {addr:#x} should be marked as used"
            );
        } else {
            assert_eq!(
                alloc.bitmap.read_bit(bit),
                Some(FREE),
                "Addr {addr:#x} should be marked as free"
            );
        }
    }
}
