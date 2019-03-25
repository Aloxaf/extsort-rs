// Copyright 2018 Andre-Philippe Paquet
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The `extsort` crate exposes external sorting (i.e. on disk sorting) capability on arbitrarily sized iterator, even if the
//! generated content of the iterator doesn't fit in memory. Once sorted, it returns a new sorted iterator.
//!
//! In order to remain efficient for all implementations, `extsort` doesn't handle serialization, but leaves that to the user.
//!
//! # Examples
//! ```rust
//! extern crate extsort;
//!
//! use extsort::*;
//!
//! let sorter = ExternalSorter::new();
//! let reversed_data = (0..1000u32).rev().into_iter();
//! let sorted_iter = sorter.sort(reversed_data).unwrap();
//! let sorted_data: Vec<u32> = sorted_iter.collect();
//!
//! let expected_data = (0..1000u32).collect::<Vec<_>>();
//! assert_eq!(sorted_data, expected_data);
//! ```

pub mod sorter;
pub use crate::sorter::{ExternalSorter, SortedIterator};
