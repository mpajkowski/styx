/// Align the nearest _lower_ aligned address
#[macro_export]
macro_rules! align_down {
    ($addr: expr, $align: expr) => {
        $addr & !($align - 1)
    };
}

/// Align the nearest _upper_ aligned address
#[macro_export]
macro_rules! align_up {
    ($addr: expr, $align:expr) => {
        ($addr + $align - 1) & !($align - 1)
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test_align_down() {
        assert_eq!(align_down!(0, 8), 0);
        assert_eq!(align_down!(1, 8), 0);
        assert_eq!(align_down!(2, 8), 0);
        assert_eq!(align_down!(3, 8), 0);
        assert_eq!(align_down!(4, 8), 0);
        assert_eq!(align_down!(5, 8), 0);
        assert_eq!(align_down!(6, 8), 0);
        assert_eq!(align_down!(7, 8), 0);
        assert_eq!(align_down!(8, 8), 8);
        assert_eq!(align_down!(9, 8), 8);
        assert_eq!(align_down!(10, 8), 8);
        assert_eq!(align_down!(11, 8), 8);
        assert_eq!(align_down!(12, 8), 8);
        assert_eq!(align_down!(13, 8), 8);
        assert_eq!(align_down!(14, 8), 8);
        assert_eq!(align_down!(15, 8), 8);
        assert_eq!(align_down!(16, 8), 16);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up!(0, 8), 0);
        assert_eq!(align_up!(1, 8), 8);
        assert_eq!(align_up!(2, 8), 8);
        assert_eq!(align_up!(3, 8), 8);
        assert_eq!(align_up!(4, 8), 8);
        assert_eq!(align_up!(5, 8), 8);
        assert_eq!(align_up!(6, 8), 8);
        assert_eq!(align_up!(7, 8), 8);
        assert_eq!(align_up!(8, 8), 8);
        assert_eq!(align_up!(9, 8), 16);
    }

    #[test]
    fn align_0x1000() {
        assert_eq!(align_down!(0x1002, 0x1000), 0x1000);
        assert_eq!(align_up!(0x1002, 0x1000), 0x2000);
    }
}
