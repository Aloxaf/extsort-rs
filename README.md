extsort [![crates.io](https://img.shields.io/crates/v/extsort.svg)](https://crates.io/crates/extsort)
==========

Exposes external sorting (i.e. on disk sorting) capability on arbitrarily sized iterator, even if the
generated content of the iterator doesn't fit in memory. Once sorted, it returns a new sorted iterator.
In order to remain efficient for all implementations, the crate doesn't handle serialization, but leaves that to the user.

# Example
```rust
extern crate extsort;

use extsort::*;

fn main() {
    let sorter = ExternalSorter::new();
    let reversed_data = (0..1000u32).rev().into_iter();
    let sorted_iter = sorter.sort(reversed_data).unwrap();
    let sorted_data: Vec<u32> = sorted_iter.collect();

    let expected_data = (0..1000u32).collect::<Vec<_>>();
    assert_eq!(sorted_data, expected_data);
}
```
