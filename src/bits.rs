use std::fmt::{Debug, Formatter};
use crate::errors::Error;
use std::mem::take;

/*
Appending bits happens right-to-left in each u64, but left-to-right in the vec. So imaging
the vec used u8 instead of u64, the memory layout would look as follows. Debug printing reverses
the order of the u64's so that you can read the bits left to right, and everything is in order
for comparing to expected results.

1. Initial:
memory: 00000000
print : 00000000

2. Append 4 ones:
memory: 00001111
print : 11110000

3. Append 5 zeros:
memory: 00001111 00000000
print : 11110000 00000000

4. Append 3 ones:
memory: 00001111 00001110
print : 11110000 01110000

 */

#[derive(Eq, PartialEq)]
pub struct Bits<V: AsRef<[u64]>> {
    bits: V,
    current_location: usize,
}

impl<V: AsRef<[u64]>> Debug for Bits<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in self.bits.as_ref().iter() {
            let s: String = format!("{:064b}", i).chars().rev().collect();
            write!(f, "{} {} ", s.chars().take(32).collect::<String>(), s.chars().skip(32).collect::<String>())?;
        }
        Ok(())
    }
}


impl Bits<Vec<u64>> {
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
        let other = other.reverse_bits() >> (64 - num_bits);

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
            *last_u64 = *last_u64 | ((other << (64 - num_bits + remaining_bits)) << self.current_location);
            self.current_location = 0;
            self.bits.push(0);
            let other = other >> remaining_bits;
            num_bits -= remaining_bits;

            // Then place the remaining bits in the new vec entry, same code as case 2
            let last_u64 = self.bits.last_mut().unwrap();
            let bit_mask = u64::MAX >> (64 - num_bits);
            *last_u64 = *last_u64 | ((other & bit_mask) << self.current_location);
            self.current_location += num_bits;

            if self.current_location == 64 {
                self.bits.push(0);
                self.current_location = 0;
            }
        }
        self
    }
}

impl<V: AsRef<[u64]>> Bits<V> {
    pub fn select_1(&self, index: usize) -> Option<usize> {
        let mut total = 0;
        for (vec_index, i) in self.bits.as_ref().iter().enumerate() {
            let c = i.count_ones() as usize;
            if total + c >= index {
                // It's in our current u64, so drop down and count
                for bit_index in 0..63 {
                    if i & (1 << bit_index) == 1 << bit_index {
                        total += 1;
                    }
                    if total == index + 1 {
                        return Some(64 * vec_index + bit_index);
                    }
                }
            }
        }
        None
    }

    pub fn select_0(&self, index: usize) -> Option<usize> {
        let mut total = 0;
        for (vec_index, i) in self.bits.as_ref().iter().enumerate() {
            let c = i.count_zeros() as usize;
            if total + c >= index {
                // It's in our current u64, so drop down and count
                for bit_index in 0..63 {
                    if ((1 << bit_index) & i) ^ (1 << bit_index) == 1 << bit_index {
                        total += 1;
                    }
                    if total == index + 1 {
                        return Some(64 * vec_index + bit_index);
                    }
                }
            }
        }
        None
    }

    pub fn slice(&self, start: usize, end: usize) -> Option<u64> {
        let started = *self.bits.as_ref().get(start / 64)? >> (start % 64);
        Some((started << (64 - (end - start))).reverse_bits())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.bits.as_ref().iter().flat_map(|i| i.to_le_bytes()).collect()
    }
}

impl<'a> Bits<&'a [u64]> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, Error> {
        if data.len() % 8 != 0 {
            return Err(Error::invalid_bits_data(data.len()));
        }
        Ok(Self {
            bits: unsafe { std::mem::transmute::<_, &[u64]>(data) },
            current_location: 0,
        })
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

    #[test]
    fn can_append_from_with_a_variety_of_bits() {
        assert_eq!(
            Bits::new().append_zeros(1).append_ones(1),
            Bits::new().append_from(1, 2)
        )
    }

    #[test]
    fn can_append_from_with_a_variety_of_bits_across_boundaries() {
        assert_eq!(
            Bits::new().append_zeros(64).append_ones(1),
            Bits::new().append_zeros(63).append_from(1, 2)
        )
    }

    #[test]
    fn select_0_in_first_u64() {
        assert_eq!(
            Bits::new().append_ones(32).append_zeros(4).select_0(3),
            Some(35)
        )
    }

    #[test]
    fn select_0_in_second_u64() {
        assert_eq!(
            Bits::new().append_ones(80).append_zeros(4).select_0(3),
            Some(83)
        )
    }

    #[test]
    fn select_1_in_first_u64() {
        assert_eq!(
            Bits::new().append_zeros(32).append_ones(4).select_1(3),
            Some(35)
        )
    }

    #[test]
    fn select_1_in_second_u64() {
        assert_eq!(
            Bits::new().append_zeros(80).append_ones(4).select_1(3),
            Some(83)
        )
    }

    #[test]
    fn select_1_real_example() {
        let mut bits = Bits::new();
        let bits = bits
            .append_ones(2)
            .append_zeros(1)
            .append_ones(2);
        assert_eq!(bits.select_1(1), Some(1));
        assert_eq!(bits.select_1(2), Some(3));
    }

    #[test]
    fn slice_in_first_u64() {
        assert_eq!(
            Bits::new().append_zeros(4).append_ones(4).slice(2, 6),
            Some(3)
        )
    }

    #[test]
    fn serialize_and_deserialize() {
        let mut bits = Bits::new();
        bits.append_zeros(4).append_ones(4);
        let data = bits.as_bytes();
        let bits = Bits::from_bytes(data.as_slice()).expect("bits");
        assert_eq!(
            bits.slice(2, 6),
            Some(3)
        )
    }
}