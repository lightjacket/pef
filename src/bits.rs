use std::fmt::{Debug, Formatter};

/*
Appending bits happens right-to-left in each u64, but left-to-right in the vec. So imaging
the vec used u8 instead of u64, the layout would look like:

1. Initial:
00000000

2. Append 4 ones:
00001111

3. Append 5 zeros:
00001111 00000000

4. Append 3 ones:
00001111 00001110

 */

#[derive(Eq, PartialEq)]
pub struct Bits {
    bits: Vec<u64>,
    current_location: usize,
}

impl Debug for Bits {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in self.bits.iter() {
            let s: String = format!("{:064b}", i).chars().rev().collect();
            write!(f, "{} {} ", s.chars().take(32).collect::<String>(), s.chars().skip(32).collect::<String>())?;
        }
        Ok(())
    }
}

impl Bits {
    pub fn new() -> Self {
        Self { bits: vec![0], current_location: 0 }
    }

    pub fn append_ones(&mut self, mut number_of_ones: usize) -> &mut Self {
        while number_of_ones > 0 {
            let to_move = number_of_ones.min(64 - self.current_location);
            let last_u64 = self.bits.last_mut().unwrap();
            *last_u64 = *last_u64 | ((u64::MAX >> (64 - to_move)) << self.current_location);
            self.current_location += to_move;
            if self.current_location >= 64 {
                self.bits.push(0);
                self.current_location %= 64;
            }
            number_of_ones = if number_of_ones >= 64 { number_of_ones - 64 } else { 0 };
        }
        self
    }

    pub fn append_zeros(&mut self, number_of_zeros: usize) -> &mut Self {
        (0..(number_of_zeros + self.current_location) / 64).for_each(|_| self.bits.push(0));
        self.current_location = (self.current_location + number_of_zeros) % 64;
        self
    }

    pub fn append_from(&mut self, other: u64, mut num_bits: usize) -> &mut Self {
        // Two cases here, (1) where the other bits overlap a boundary in our vec, and (2) where
        // it does not
        if self.current_location + num_bits <= 64 {
            // no boundary overlap (case 2)
            let last_u64 = self.bits.last_mut().unwrap();
            let bit_mask = u64::MAX >> (64 - num_bits);
            *last_u64 = *last_u64 | ((other & bit_mask) << self.current_location);
            self.current_location += num_bits;

            if self.current_location == 64 {
                self.bits.push(0);
                self.current_location = 0;
            }
        } else {
            // boundary overlap (case 1)
            // first fill current bit vec index
            let last_u64 = self.bits.last_mut().unwrap();
            let remaining_bits = 64 - self.current_location;
            *last_u64 = *last_u64 | ((other >> (num_bits - remaining_bits)) << self.current_location);
            self.current_location = 0;
            self.bits.push(0);
            num_bits -= remaining_bits;

            // Then place the remaining bits in the new vec entry, same code as case 2
            let last_u64 = self.bits.last_mut().unwrap();
            *last_u64 = *last_u64 | (other << self.current_location);
            self.current_location += num_bits;

            if self.current_location == 64 {
                self.bits.push(0);
                self.current_location = 0;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_append_bits_on_first_64_bits() {
        assert_eq!(Bits::new().append_from(7, 3), Bits::new().append_ones(3));
    }

    #[test]
    fn can_append_bits_on_more_than_64_bits() {
        let mut a = Bits::new();
        a.append_from(u64::MAX, 64).append_from(u64::MAX, 64);
        let mut b = Bits::new();
        b.append_ones(128);
        assert_eq!(a, b);
    }

    #[test]
    fn can_append_bits_overlapping_boundaries() {
        let mut a = Bits::new();
        a
            .append_from((u64::MAX << 24) >> 24, 40)
            .append_from(0, 40)
            .append_from((u64::MAX << 44) >> 44, 20);
        let mut b = Bits::new();
        b
            .append_ones(40)
            .append_zeros(40)
            .append_ones(20);
        assert_eq!(a, b);
    }
}