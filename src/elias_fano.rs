use crate::bits::Bits;
use crate::errors::Error;


// Logic taken from https://www.antoniomallia.it/sorted-integers-compression-with-elias-fano-encoding.html
#[derive(Debug)]
pub struct EliasFano {
    upper_bits: Bits,
    lower_bits: Bits,
    num_lower_bits: usize,
    num_upper_bits: usize,
}

impl EliasFano {
    pub fn new(ids: Vec<usize>) -> Result<Self, Error> {
        if !ids.iter().zip(ids.iter().skip(1)).all(|(a, b)| a < b) {
            return Err(Error::unsorted_ids());
        }

        let m = ids.last().ok_or(Error::no_ids())?;
        let n = ids.len();
        let num_lower_bits = ((*m as f64) / (n as f64)).log2().ceil() as usize;
        let lower_mask = u64::MAX >> (64 - num_lower_bits);
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
        })
    }

    pub fn get(&self, index: usize) -> Option<usize> {
        let lower = self.lower_bits.slice(
            index * self.num_lower_bits,
            (index + 1) * self.num_lower_bits,
        )?;
        let upper = self.upper_bits.select_1(index)? - index;
        println!("lower={} upper={}", lower, upper);
        Some((((upper as u64) << self.num_lower_bits) | lower) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let ef = EliasFano::new(vec![2, 3, 5, 7, 11, 13, 24]).expect("elias fano encoding");
        assert_eq!(ef.get(0), Some(2));
        assert_eq!(ef.get(1), Some(3));
        assert_eq!(ef.get(4), Some(11));
        assert_eq!(ef.get(6), Some(24));
    }
}