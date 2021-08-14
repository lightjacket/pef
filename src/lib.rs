mod bits;
mod elias_fano;
mod errors;

#[cfg(test)]
mod tests {
    use flate2::Compression;
    use std::io::Write;
    use crate::elias_fano::EliasFano;

    #[test]
    fn it_works() {
        let ids: Vec<usize> = (0..100).chain(1000..1023).chain(1060..1400).chain(20000..20001).collect();

        let gzipped = {
            let ids: Vec<_> = ids.iter().flat_map(|i| (*i as u32).to_be_bytes()).collect();
            let mut out = vec![];
            let mut encoder = flate2::write::GzEncoder::new(&mut out, Compression::new(9));
            encoder.write(ids.as_slice()).expect("wrote bytes");
            encoder.finish().expect("finished");
            out.len()
        };

        let efed = {
            let ef = EliasFano::new(ids).expect("elias fano encoding");
            ef.as_bytes().len()
        };

        println!("ef={} gzip={}", efed, gzipped);
        assert!(efed < gzipped, "elias fano size is better");
    }
}
