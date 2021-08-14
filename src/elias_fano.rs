use crate::bits::Bits;
use crate::errors::Error;


// Logic taken from https://www.antoniomallia.it/sorted-integers-compression-with-elias-fano-encoding.html
#[derive(Debug)]
pub struct EliasFano {
    upper_bits: Bits,
    lower_bits: Bits,
    num_lower_bits: usize,
    num_upper_bits: usize,
    size: usize,
}

impl EliasFano {
    pub fn new(ids: Vec<usize>) -> Result<Self, Error> {
        let size = ids.len();

        if !ids.iter().zip(ids.iter().skip(1)).all(|(a, b)| a < b) {
            return Err(Error::unsorted_ids());
        }

        let m = ids.last().ok_or(Error::no_ids())?;
        let n = ids.len();
        let num_lower_bits = ((*m as f64) / (n as f64)).log2().ceil() as usize;
        let num_upper_bits = (n as f64).log2().ceil() as usize;
        let upper_mask = u64::MAX >> (64 - num_upper_bits);

        let mut all_lower_bits = Bits::new();

        let mut upper_bit_buckets: Vec<_> = (0..upper_mask).map(|_| 0).collect();

        for id in ids {
            all_lower_bits.append_from(id as u64, num_lower_bits);
            let upper_bits = ((id as u64) >> num_lower_bits) & upper_mask;
            upper_bit_buckets[upper_bits as usize] += 1;
        }

        let mut all_upper_bits = Bits::new();
        for bucket_count in upper_bit_buckets {
            all_upper_bits.append_ones(bucket_count);
            all_upper_bits.append_zeros(1);
        }

        Ok(Self {
            lower_bits: all_lower_bits,
            upper_bits: all_upper_bits,
            num_lower_bits,
            num_upper_bits,
            size,
        })
    }

    pub fn get(&self, index: usize) -> Option<usize> {
        let lower = self.lower_bits.slice(
            index * self.num_lower_bits,
            (index + 1) * self.num_lower_bits,
        )?;
        let upper = self.upper_bits.select_1(index)? - index;
        Some((((upper as u64) << self.num_lower_bits) | lower) as usize)
    }

    pub fn next_geq(&self, value: usize) -> Option<usize> {
        let value = value as u64;
        let upper_bits_bucket = (value >> self.num_lower_bits) & (u64::MAX >> (64 - self.num_upper_bits));

        let start = if upper_bits_bucket == 0 {
            0
        } else {
            self.upper_bits.select_0(upper_bits_bucket as usize - 1)? - upper_bits_bucket as usize
        };

        for i in start..self.size {
            let to_check = self.get(i);
            if to_check >= Some(value as usize) {
                return to_check;
            }
        }
        None
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = (self.size as u64).to_be_bytes().to_vec();
        vec.append(&mut (self.num_upper_bits as u64).to_be_bytes().to_vec());
        vec.append(&mut (self.num_lower_bits as u64).to_be_bytes().to_vec());
        vec.append(&mut self.upper_bits.as_bytes());
        vec.append(&mut self.lower_bits.as_bytes());
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ef_get() {
        let ef = EliasFano::new(vec![2, 3, 5, 7, 11, 13, 24]).expect("elias fano encoding");
        assert_eq!(ef.get(0), Some(2));
        assert_eq!(ef.get(1), Some(3));
        assert_eq!(ef.get(4), Some(11));
        assert_eq!(ef.get(6), Some(24));
    }

    #[test]
    fn ef_next_geq() {
        let ef = EliasFano::new(vec![2, 3, 5, 7, 11, 13, 24]).expect("elias fano encoding");
        assert_eq!(ef.next_geq(1), Some(2));
        assert_eq!(ef.next_geq(3), Some(3));
        assert_eq!(ef.next_geq(4), Some(5));
        assert_eq!(ef.next_geq(14), Some(24));
    }
}