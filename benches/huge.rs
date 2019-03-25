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

#![feature(test)]
extern crate test;

use extsort::*;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_vec_sort_1000(b: &mut Bencher) {
        b.iter(|| {
            let mut sorted_iter: Vec<u32> =
                (0..1000).into_iter().rev().collect();
            sorted_iter.sort();
        })
    }

    #[bench]
    fn bench_ext_sort_1000(b: &mut Bencher) {
        let sorter = ExternalSorter::new();
        b.iter(|| {
            let sorted_iter = sorter
                .sort((0..1000u32).into_iter().rev())
                .unwrap();
            sorted_iter.sorted_count();
        })
    }

    #[bench]
    fn bench_vec_sort_100_000(b: &mut Bencher) {
        b.iter(|| {
            let mut sorted_iter: Vec<u32> =
                (0..100_000).into_iter().rev().collect();
            sorted_iter.sort();
        })
    }

    #[bench]
    fn bench_ext_sort_100_000(b: &mut Bencher) {
        let sorter = ExternalSorter::new();
        b.iter(|| {
            let sorted_iter = sorter
                .sort((0..100_000u32).into_iter().rev())
                .unwrap();
            sorted_iter.sorted_count();
        })
    }

    #[bench]
    fn bench_ext_sort_1million(b: &mut Bencher) {
        let mut sorter = ExternalSorter::new();
        sorter.set_max_size(100_000);

        b.iter(|| {
            let sorted_iter = sorter
                .sort((0..1_000_000u32).into_iter().rev())
                .unwrap();
            sorted_iter.sorted_count();
        })
    }
}
