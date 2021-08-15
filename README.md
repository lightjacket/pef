# PEF
This repository is an ongoing Rust implementation of the Partitioned Elias-Fano
encoding for sorted integers. The status is as follows:

- [x] Working Elias-Fano implementation (can encode and get the bytes for the representation)
- [ ] Partitioned Elias-Fano implementation
- [ ] Read performance increases

## Examples
```rust
let ef = EliasFano::new(vec![1,2,5]);
assert_eq!(ef.get(1), Some(2));
assert_eq!(ef.next_geq(4), Some(5));
let serialized = ef.as_bytes();
```