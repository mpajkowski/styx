use core::ops::Range;

pub trait BitManipulate: Sized {
    const BIT_LEN: usize;

    fn read_bit(&self, bit: usize) -> bool;
    fn set_bit(&mut self, bit: usize, value: bool);
    fn read_range(&self, range: Range<usize>) -> Self;
    fn set_range(&mut self, range: Range<usize>, value: Self);
}

macro_rules! impl_bit_manipulate {
    ($($t: ty),*) => {
        $(
        impl BitManipulate for $t {
            const BIT_LEN: usize = core::mem::size_of::<$t>() * 8;

            #[inline]
            fn read_bit(&self, bit: usize) -> bool {
                debug_assert!(bit < Self::BIT_LEN);

                ((self >> bit) & 1) != 0
            }

            #[inline]
            fn set_bit(&mut self, bit: usize, value: bool) {
                debug_assert!(bit < Self::BIT_LEN);

                if value {
                    *self |= 1 << bit;
                } else {
                    *self &= !(1 << bit);
                }
            }

            #[inline]
            fn read_range(&self, range: Range<usize>) -> Self {
                debug_assert!(range.start < Self::BIT_LEN);
                debug_assert!(range.end <= Self::BIT_LEN);
                debug_assert!(range.end > range.start);

                let high_shift = Self::BIT_LEN - range.end;
                (self << high_shift >> high_shift) >> range.start
            }

            #[inline]
            fn set_range(&mut self, range: Range<usize>, value: Self) {
                debug_assert!(range.start < Self::BIT_LEN);
                debug_assert!(range.end <= Self::BIT_LEN);
                debug_assert!(range.end > range.start);

                let high_shift = Self::BIT_LEN - range.end;
                let mask = !(!(0 << high_shift) >> high_shift >> range.start << range.start);

                *self = (*self & mask) | (value << range.start)
            }
        }
        )*
    };
}

impl_bit_manipulate!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_bits() {
        let val: u8 = 0b0000_1111;

        assert!(val.read_bit(0));
        assert!(val.read_bit(1));
        assert!(val.read_bit(2));
        assert!(val.read_bit(3));
        assert!(!val.read_bit(4));
        assert!(!val.read_bit(5));
        assert!(!val.read_bit(6));
        assert!(!val.read_bit(7));
    }

    #[test]
    fn set_bits() {
        let mut val: u8 = 0b0000_0000;

        assert!(!val.read_bit(0));
        val.set_bit(0, true);
        assert!(val.read_bit(0));
        assert_eq!(val, 0b0000_0001);

        assert!(!val.read_bit(1));
        val.set_bit(1, true);
        assert!(val.read_bit(1));
        assert_eq!(val, 0b0000_0011);

        assert!(!val.read_bit(2));
        val.set_bit(2, true);
        assert!(val.read_bit(2));
        assert_eq!(val, 0b0000_0111);

        assert!(!val.read_bit(3));
        val.set_bit(3, true);
        assert!(val.read_bit(3));
        assert_eq!(val, 0b0000_1111);

        assert!(!val.read_bit(4));
        val.set_bit(4, true);
        assert!(val.read_bit(4));
        assert_eq!(val, 0b0001_1111);

        assert!(!val.read_bit(5));
        val.set_bit(5, true);
        assert!(val.read_bit(5));
        assert_eq!(val, 0b0011_1111);

        assert!(!val.read_bit(6));
        val.set_bit(6, true);
        assert!(val.read_bit(6));
        assert_eq!(val, 0b0111_1111);

        assert!(!val.read_bit(7));
        val.set_bit(7, true);
        assert!(val.read_bit(7));
        assert_eq!(val, 0b1111_1111);

        assert!(val.read_bit(0));
        val.set_bit(0, false);
        assert!(!val.read_bit(0));
        assert_eq!(val, 0b1111_1110);

        assert!(val.read_bit(1));
        val.set_bit(1, false);
        assert!(!val.read_bit(1));
        assert_eq!(val, 0b1111_1100);

        assert!(val.read_bit(2));
        val.set_bit(2, false);
        assert!(!val.read_bit(2));
        assert_eq!(val, 0b1111_1000);

        assert!(val.read_bit(3));
        val.set_bit(3, false);
        assert!(!val.read_bit(3));
        assert_eq!(val, 0b1111_0000);

        assert!(val.read_bit(4));
        val.set_bit(4, false);
        assert!(!val.read_bit(4));
        assert_eq!(val, 0b1110_0000);

        assert!(val.read_bit(5));
        val.set_bit(5, false);
        assert!(!val.read_bit(5));
        assert_eq!(val, 0b1100_0000);

        assert!(val.read_bit(6));
        val.set_bit(6, false);
        assert!(!val.read_bit(6));
        assert_eq!(val, 0b1000_0000);

        assert!(val.read_bit(7));
        val.set_bit(7, false);
        assert!(!val.read_bit(7));
        assert_eq!(val, 0b0000_0000);
    }

    #[test]
    fn read_range() {
        let val: u8 = 0b0000_1111;

        assert_eq!(val.read_range(0..4), 0b1111);
        assert_eq!(val.read_range(1..5), 0b0111);
        assert_eq!(val.read_range(2..6), 0b0011);
        assert_eq!(val.read_range(3..7), 0b0001);
        assert_eq!(val.read_range(4..8), 0b0000);
    }

    #[test]
    fn set_range() {
        let mut val = 0b0000_1111;

        val.set_range(0..4, 0);
        assert_eq!(val, 0b0000_0000, "{:#b}", val);

        val.set_range(4..8, 0b1111);
        assert_eq!(val, 0b1111_0000, "{:#b}", val);
    }
}
