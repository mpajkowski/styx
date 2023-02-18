use core::fmt::Debug;

use easybit::*;

pub struct Bitmap<'a> {
    storage: &'a mut [u8],
}

impl Debug for Bitmap<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in &self.storage[..self.storage.len() - 1] {
            write!(f, "{:08b} ", byte.reverse_bits())?;
        }

        write!(
            f,
            "{:08b}",
            self.storage.last().copied().unwrap().reverse_bits()
        )?;

        Ok(())
    }
}

impl<'a> Bitmap<'a> {
    pub fn new(storage: &'a mut [u8]) -> Self {
        Self { storage }
    }

    pub fn fill(&mut self, value: bool) {
        let fill = if value { u8::MAX } else { 0 };
        self.storage.fill(fill);
    }

    pub fn set_bit(&mut self, pos: usize, value: bool) -> Option<()> {
        let (byte_idx, bit_idx) = self.index(pos)?;

        self.storage[byte_idx] = set_bit!(self.storage[byte_idx], bit_idx, value);

        Some(())
    }

    pub fn read_bit(&self, pos: usize) -> Option<bool> {
        let (byte_idx, bit_idx) = self.index(pos)?;

        self.storage
            .get(byte_idx)
            .map(|byte| read_bit!(*byte, bit_idx))
    }

    pub fn find_first(&self, start: Option<usize>, len: usize, value: bool) -> Option<usize> {
        let mut curr = start.unwrap_or_default();
        let (byte_idx, bit_idx) = self.index(curr)?;

        let mut found = 0;

        let mut check_bit = |byte, bit| {
            curr += 1;

            if read_bit!(byte, bit) == value {
                found += 1;
            } else {
                found = 0;
            }

            if found == len {
                let start = curr - len;
                Some(start)
            } else {
                None
            }
        };

        // check first byte
        if let found @ Some(_) = (bit_idx..8).find_map(|bit| check_bit(self.storage[byte_idx], bit))
        {
            return found;
        }

        // check first byte
        self.storage
            .get(byte_idx + 1..)?
            .iter()
            .flat_map(|byte| (0..8).map(|bit| (*byte, bit as usize)))
            .find_map(|(byte, bit)| check_bit(byte, bit))
    }

    const fn index(&self, idx: usize) -> Option<(usize, usize)> {
        let byte_idx = idx / 8;

        if byte_idx < self.storage.len() {
            Some((byte_idx, idx % 8))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(true, 0xff)]
    #[case(false, 0x00)]
    fn fill(#[case] fill_value: bool, #[case] expected: u8) {
        let storage: &mut [u8] = &mut [0b11110000, 0b00001111];
        let mut bitmap = Bitmap::new(storage);

        bitmap.fill(fill_value);
        assert!(storage.iter().all(|b| *b == expected));
    }

    #[rstest]
    #[case(0, Some(false))]
    #[case(1, Some(false))]
    #[case(2, Some(false))]
    #[case(3, Some(false))]
    #[case(4, Some(true))]
    #[case(5, Some(true))]
    #[case(6, Some(true))]
    #[case(7, Some(true))]
    #[case(8, Some(true))]
    #[case(9, Some(true))]
    #[case(10, Some(true))]
    #[case(11, Some(true))]
    #[case(12, Some(false))]
    #[case(13, Some(false))]
    #[case(14, Some(false))]
    #[case(15, Some(false))]
    #[case(16, None)]
    fn read_bit(#[case] position: usize, #[case] expected: Option<bool>) {
        let storage: &mut [u8] = &mut [0b11110000, 0b00001111];
        let bitmap = Bitmap::new(storage);

        assert_eq!(bitmap.read_bit(position), expected);
    }

    #[test]
    fn set_bit() {
        let storage: &mut [u8] = &mut [0b11110000, 0b00001111];
        let mut bitmap = Bitmap::new(storage);

        for i in 0..16 {
            let bit = bitmap.read_bit(i).unwrap();
            bitmap.set_bit(i, !bit).unwrap();
        }

        assert_eq!(storage, [0b00001111, 0b11110000]);
    }

    #[rstest]
    #[case(None, 1, Some(4))]
    #[case(None, 2, Some(4))]
    #[case(None, 3, Some(4))]
    #[case(None, 4, Some(4))]
    #[case(None, 5, Some(4))]
    #[case(None, 6, Some(4))]
    #[case(None, 7, Some(4))]
    #[case(None, 8, Some(4))]
    #[case(None, 9, None)]
    #[case(Some(1), 1, Some(4))]
    #[case(Some(1), 2, Some(4))]
    #[case(Some(1), 3, Some(4))]
    #[case(Some(1), 4, Some(4))]
    #[case(Some(1), 5, Some(4))]
    #[case(Some(1), 6, Some(4))]
    #[case(Some(1), 7, Some(4))]
    #[case(Some(1), 8, Some(4))]
    #[case(Some(1), 9, None)]
    #[case(Some(2), 1, Some(4))]
    #[case(Some(2), 2, Some(4))]
    #[case(Some(2), 3, Some(4))]
    #[case(Some(2), 4, Some(4))]
    #[case(Some(2), 5, Some(4))]
    #[case(Some(2), 6, Some(4))]
    #[case(Some(2), 7, Some(4))]
    #[case(Some(2), 8, Some(4))]
    #[case(Some(2), 9, None)]
    #[case(Some(3), 1, Some(4))]
    #[case(Some(3), 2, Some(4))]
    #[case(Some(3), 3, Some(4))]
    #[case(Some(3), 4, Some(4))]
    #[case(Some(3), 5, Some(4))]
    #[case(Some(3), 6, Some(4))]
    #[case(Some(3), 7, Some(4))]
    #[case(Some(3), 8, Some(4))]
    #[case(Some(3), 9, None)]
    #[case(Some(4), 1, Some(4))]
    #[case(Some(4), 2, Some(4))]
    #[case(Some(4), 3, Some(4))]
    #[case(Some(4), 4, Some(4))]
    #[case(Some(4), 5, Some(4))]
    #[case(Some(4), 6, Some(4))]
    #[case(Some(4), 7, Some(4))]
    #[case(Some(4), 8, Some(4))]
    #[case(Some(4), 9, None)]
    #[case(Some(5), 1, Some(5))]
    #[case(Some(5), 2, Some(5))]
    #[case(Some(5), 3, Some(5))]
    #[case(Some(5), 4, Some(5))]
    #[case(Some(5), 5, Some(5))]
    #[case(Some(5), 6, Some(5))]
    #[case(Some(5), 7, Some(5))]
    #[case(Some(5), 8, None)]
    #[case(Some(6), 1, Some(6))]
    #[case(Some(6), 2, Some(6))]
    #[case(Some(6), 3, Some(6))]
    #[case(Some(6), 4, Some(6))]
    #[case(Some(6), 5, Some(6))]
    #[case(Some(6), 6, Some(6))]
    #[case(Some(6), 7, None)]
    #[case(Some(7), 1, Some(7))]
    #[case(Some(7), 2, Some(7))]
    #[case(Some(7), 3, Some(7))]
    #[case(Some(7), 4, Some(7))]
    #[case(Some(7), 5, Some(7))]
    #[case(Some(7), 7, None)]
    #[case(Some(8), 1, Some(8))]
    #[case(Some(8), 2, Some(8))]
    #[case(Some(8), 3, Some(8))]
    #[case(Some(8), 4, Some(8))]
    #[case(Some(8), 5, None)]
    #[case(Some(9), 1, Some(9))]
    #[case(Some(9), 2, Some(9))]
    #[case(Some(9), 3, Some(9))]
    #[case(Some(9), 4, None)]
    #[case(Some(10), 1, Some(10))]
    #[case(Some(10), 2, Some(10))]
    #[case(Some(10), 3, None)]
    #[case(Some(11), 1, Some(11))]
    #[case(Some(11), 2, None)]
    #[case(Some(12), 1, None)]
    fn find_first_free(
        #[case] start_position: Option<usize>,
        #[case] len: usize,
        #[case] expected: Option<usize>,
    ) {
        let storage: &mut [u8] = &mut [0b00001111, 0b11110000];
        let bitmap = Bitmap::new(storage);

        assert_eq!(bitmap.find_first(start_position, len, false), expected);
    }
}
