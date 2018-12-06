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

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Error, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use tempdir;

pub struct ExternalSorter {
    max_size: usize,
    sort_dir: Option<PathBuf>,
}

impl ExternalSorter {
    pub fn new() -> ExternalSorter {
        ExternalSorter {
            max_size: 10000,
            sort_dir: None,
        }
    }

    /// Set maximum number of items we can buffer in memory
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
    }

    /// Set directory in which sorted segments will be written (if it doesn't fit in memory)
    pub fn set_sort_dir(&mut self, path: PathBuf) {
        self.sort_dir = Some(path);
    }

    /// Sort a given iterator, returning a new iterator with items
    pub fn sort<T, I>(&self, mut iterator: I) -> Result<SortedIterator<T>, Error>
    where
        T: Sortable<T>,
        I: Iterator<Item = T>,
    {
        let mut tempdir: Option<tempdir::TempDir> = None;
        let sort_dir = if let Some(ref sort_dir) = self.sort_dir {
            sort_dir.to_path_buf()
        } else {
            tempdir = Some(tempdir::TempDir::new("sort")?);
            tempdir.as_ref().unwrap().path().to_path_buf()
        };

        let mut segments: Vec<File> = Vec::new();
        let mut buffer: Vec<T> = Vec::new();
        loop {
            let next_item = iterator.next();
            if next_item.is_none() {
                break;
            }

            buffer.push(next_item.unwrap());
            if buffer.len() > self.max_size {
                Self::sort_and_write_segment(&sort_dir, &mut segments, &mut buffer)?;
                buffer.clear();
            }
        }

        // Write any items left in buffer, but only if we had at least 1 segment writen.
        // Otherwise we use the buffer itself to iterate from memory
        let pass_through_queue = if !buffer.is_empty() && !segments.is_empty() {
            Self::sort_and_write_segment(&sort_dir, &mut segments, &mut buffer)?;
            None
        } else {
            buffer.sort();
            Some(VecDeque::from(buffer))
        };

        SortedIterator::new(tempdir, pass_through_queue, segments)
    }

    fn sort_and_write_segment<T>(
        sort_dir: &Path,
        segments: &mut Vec<File>,
        buffer: &mut [T],
    ) -> Result<(), Error>
    where
        T: Sortable<T>,
    {
        buffer.sort();

        let segment_path = sort_dir.join(format!("{}", segments.len()));
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&segment_path)?;
        let mut buf_writer = BufWriter::new(file);
        for item in buffer {
            <T as Sortable<T>>::encode(item, &mut buf_writer);
        }

        let file = buf_writer.into_inner()?;
        segments.push(file);

        Ok(())
    }
}

impl Default for ExternalSorter {
    fn default() -> Self {
        ExternalSorter::new()
    }
}

pub trait Sortable<T>: Eq + Ord {
    fn encode(item: &T, write: &mut Write);
    fn decode(read: &mut Read) -> Option<T>;
}

pub struct SortedIterator<T: Sortable<T>> {
    _tempdir: Option<tempdir::TempDir>,
    pass_through_queue: Option<VecDeque<T>>,
    segments: Vec<BufReader<File>>,
    next_values: Vec<Option<T>>,
}

impl<T: Sortable<T>> SortedIterator<T> {
    fn new(
        tempdir: Option<tempdir::TempDir>,
        pass_through_queue: Option<VecDeque<T>>,
        mut segments: Vec<File>,
    ) -> Result<SortedIterator<T>, Error> {
        for segment in &mut segments {
            segment.seek(SeekFrom::Start(0))?;
        }

        let next_values = segments
            .iter_mut()
            .map(|file| Self::read_item(file))
            .collect();

        let segments = segments.into_iter().map(BufReader::new).collect();

        Ok(SortedIterator {
            _tempdir: tempdir,
            pass_through_queue,
            segments,
            next_values,
        })
    }

    fn read_item(file: &mut Read) -> Option<T> {
        <T as Sortable<T>>::decode(file)
    }
}

impl<T: Sortable<T>> Iterator for SortedIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        // if we have a pass through, we dequeue from it directly
        if let Some(ptb) = self.pass_through_queue.as_mut() {
            return ptb.pop_front();
        }

        // otherwise, we iter from segments on disk
        let mut smallest_idx: Option<usize> = None;
        {
            let mut smallest: Option<&T> = None;
            for idx in 0..self.segments.len() {
                let next_value = self.next_values[idx].as_ref();
                if next_value.is_none() {
                    continue;
                }

                if smallest.is_none() || *next_value.unwrap() < *smallest.unwrap() {
                    smallest = Some(next_value.unwrap());
                    smallest_idx = Some(idx);
                }
            }
        }

        match smallest_idx {
            Some(idx) => {
                let file = &mut self.segments[idx];
                let value = self.next_values[idx].take().unwrap();
                self.next_values[idx] = Self::read_item(file);
                Some(value)
            }
            None => None,
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use self::byteorder::{ReadBytesExt, WriteBytesExt};

    pub extern crate byteorder;

    #[test]
    fn test_smaller_than_segment() {
        let sorter = ExternalSorter::new();
        let data: Vec<u32> = (0..100u32).collect();
        let data_rev: Vec<u32> = data.iter().rev().cloned().collect();

        let sorted_iter = sorter.sort(data_rev.into_iter()).unwrap();

        // should not have used any segments (all in memory)
        assert_eq!(sorted_iter.segments.len(), 0);
        let sorted_data: Vec<u32> = sorted_iter.collect();

        assert_eq!(data, sorted_data);
    }

    #[test]
    fn test_multiple_segments() {
        let mut sorter = ExternalSorter::new();
        sorter.set_max_size(100);
        let data: Vec<u32> = (0..1000u32).collect();

        let data_rev: Vec<u32> = data.iter().rev().cloned().collect();
        let sorted_iter = sorter.sort(data_rev.into_iter()).unwrap();
        assert_eq!(sorted_iter.segments.len(), 10);

        let sorted_data: Vec<u32> = sorted_iter.collect();
        assert_eq!(data, sorted_data);
    }

    impl Sortable<u32> for u32 {
        fn encode(item: &u32, write: &mut Write) {
            write.write_u32::<byteorder::LittleEndian>(*item).unwrap();
        }

        fn decode(read: &mut Read) -> Option<u32> {
            read.read_u32::<byteorder::LittleEndian>().ok()
        }
    }
}