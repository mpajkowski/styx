#[macro_export]
macro_rules! bit_sizeof {
    ($target: expr) => {{
        core::mem::size_of_val(&$target) * 8
    }};
}

#[macro_export]
macro_rules! read_bit {
    ($target: expr, $bit: expr) => {{
        (($target >> $bit) & 1) != 0
    }};
}

#[macro_export]
macro_rules! set_bit {
    ($target: expr, $bit: expr, $value: expr) => {{
        if $value {
            $target | 1 << $bit
        } else {
            $target & !(1 << $bit)
        }
    }};

    ($target: expr, $bit: expr, true) => {{
        $target | 1 << $bit
    }};

    ($target: expr, $bit: expr, false) => {{
        $target & !(1 << $bit)
    }};
}

#[macro_export]
macro_rules! read_range {
    ($target: expr, $range:expr) => {{
        let high_shift = bit_sizeof!($target) - $range.end;

        ($target << high_shift >> high_shift) >> $range.start
    }};
}

#[macro_export]
macro_rules! set_range {
    ($target: expr, $range:expr, $value: expr) => {{
        let high_shift = bit_sizeof!($target) - $range.end;

        let mask = !(!(0 << high_shift) >> high_shift >> $range.start << $range.start);

        ($target & mask) | ($value << $range.start)
    }};
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_bits() {
        let val: u8 = 0b0000_1111;

        assert!(read_bit!(val, 0));
        assert!(read_bit!(val, 1));
        assert!(read_bit!(val, 2));
        assert!(read_bit!(val, 3));
        assert!(!read_bit!(val, 4));
        assert!(!read_bit!(val, 5));
        assert!(!read_bit!(val, 6));
        assert!(!read_bit!(val, 7));
    }

    #[test]
    fn set_bits() {
        let mut val: u8 = 0b0000_0000;

        assert!(!read_bit!(val, 0));
        val = set_bit!(val, 0, true);
        assert!(read_bit!(val, 0));
        assert_eq!(val, 0b0000_0001);

        assert!(!read_bit!(val, 1));
        val = set_bit!(val, 1, true);
        assert!(read_bit!(val, 1));
        assert_eq!(val, 0b0000_0011);

        assert!(!read_bit!(val, 2));
        val = set_bit!(val, 2, true);
        assert!(read_bit!(val, 2));
        assert_eq!(val, 0b0000_0111);

        assert!(!read_bit!(val, 3));
        val = set_bit!(val, 3, true);
        assert!(read_bit!(val, 3));
        assert_eq!(val, 0b0000_1111);

        assert!(!read_bit!(val, 4));
        val = set_bit!(val, 4, true);
        assert!(read_bit!(val, 4));
        assert_eq!(val, 0b0001_1111);

        assert!(!read_bit!(val, 5));
        val = set_bit!(val, 5, true);
        assert!(read_bit!(val, 5));
        assert_eq!(val, 0b0011_1111);

        assert!(!read_bit!(val, 6));
        val = set_bit!(val, 6, true);
        assert!(read_bit!(val, 6));
        assert_eq!(val, 0b0111_1111);

        assert!(!read_bit!(val, 7));
        val = set_bit!(val, 7, true);
        assert!(read_bit!(val, 7));
        assert_eq!(val, 0b1111_1111);

        assert!(read_bit!(val, 0));
        val = set_bit!(val, 0, false);
        assert!(!read_bit!(val, 0));
        assert_eq!(val, 0b1111_1110);

        assert!(read_bit!(val, 1));
        val = set_bit!(val, 1, false);
        assert!(!read_bit!(val, 1));
        assert_eq!(val, 0b1111_1100);

        assert!(read_bit!(val, 2));
        val = set_bit!(val, 2, false);
        assert!(!read_bit!(val, 2));
        assert_eq!(val, 0b1111_1000);

        assert!(read_bit!(val, 3));
        val = set_bit!(val, 3, false);
        assert!(!read_bit!(val, 3));
        assert_eq!(val, 0b1111_0000);

        assert!(read_bit!(val, 4));
        val = set_bit!(val, 4, false);
        assert!(!read_bit!(val, 4));
        assert_eq!(val, 0b1110_0000);

        assert!(read_bit!(val, 5));
        val = set_bit!(val, 5, false);
        assert!(!read_bit!(val, 5));
        assert_eq!(val, 0b1100_0000);

        assert!(read_bit!(val, 6));
        val = set_bit!(val, 6, false);
        assert!(!read_bit!(val, 6));
        assert_eq!(val, 0b1000_0000);

        assert!(read_bit!(val, 7));
        val = set_bit!(val, 7, false);
        assert!(!read_bit!(val, 7));
        assert_eq!(val, 0b0000_0000);
    }

    pub fn set_dynamic() {
        let mut target = 0_u8;
        let value = true;

        target = set_bit!(target, 0, value);
    }

    #[test]
    fn read_range() {
        let val: u8 = 0b0000_1111;

        assert_eq!(read_range!(val, 0..4), 0b1111);
        assert_eq!(read_range!(val, 1..5), 0b0111);
        assert_eq!(read_range!(val, 2..6), 0b0011);
        assert_eq!(read_range!(val, 3..7), 0b0001);
        assert_eq!(read_range!(val, 4..8), 0b0000);
    }

    #[test]
    fn set_range() {
        let mut val = 0b0000_1111;

        val = set_range!(val, 0..4, 0);
        assert_eq!(val, 0b0000_0000, "{val:#b}");

        val = set_range!(val, 4..8, 0b1111);
        assert_eq!(val, 0b1111_0000, "{val:#b}");
    }

    #[test]
    fn allow_in_const_context() {
        const fn const_fn() -> bool {
            let on = true;
            let off = false;
            let mut x = read_bit!(0x1_u8, 0);
            let mut x = set_bit!(0x1_u8, 0, on);
            let mut x = set_bit!(0x1_u8, 0, off);
            x == 0
        }
        assert!(const_fn());
    }
}
