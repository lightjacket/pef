#[derive(Eq, PartialEq, Debug)]
pub struct Bits {
    bits: Vec<u64>,
    current_location: usize,
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
        self.current_location = (self.current_location + number_of_zeros) % 64;
        (0..(number_of_zeros + self.current_location) / 64).for_each(|_| self.bits.push(0));
        self
    }

    pub fn append_from(&mut self, other: u64, mut num_bits: usize) -> &mut Self {
        // Two cases here, (1) where the other bits overlap a boundary in our vec, and (2) where
        // it does not
        if self.current_location + num_bits <= 64 {
            // no boundary overlap (case 2)
            let last_u64 = self.bits.last_mut().unwrap();
            *last_u64 = *last_u64 | (other << self.current_location);
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
}