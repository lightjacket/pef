#[derive(Eq, PartialEq, Debug)]
pub struct Bits {
    bits: Vec<u64>,
    current_location: usize
}

impl Bits {
    pub fn new() -> Self {
        Self { bits: vec![0], current_location: 0 }
    }

    pub fn append_ones(&mut self, number_of_ones: usize) -> &mut Self {
        let last_u64 = self.bits.last_mut().unwrap();
        *last_u64 = *last_u64 | ((u64::MAX >> (64 - number_of_ones)) << self.current_location);
        self.current_location += number_of_ones;
        self
    }

    pub fn append_zeros(&mut self, number_of_zeros: usize) -> &mut Self {
        self.current_location += number_of_zeros;
        self
    }

    pub fn append_from(&mut self, other: u64, num_bits: usize) -> &mut Self {
        let last_u64 = self.bits.last_mut().unwrap();
        *last_u64 = *last_u64 | (other << self.current_location);
        self.current_location += num_bits;
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
}