use std::{collections::BinaryHeap, iter};

use crate::{
    engine_error::EngineError,
    skiplist::SkipListKV,
    sstable::{iterator::SstableIterator, writer::SstableWriter},
};

struct HeapItem<I: Iterator> {
    next_item: I::Item,
    iter: I,
}

impl Ord for HeapItem<SstableIterator> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .next_item
            .key
            .cmp(&self.next_item.key)
            .then_with(|| self.next_item.sequence.cmp(&other.next_item.sequence))
    }
}
impl Eq for HeapItem<SstableIterator> {}
impl PartialEq for HeapItem<SstableIterator> {
    fn eq(&self, other: &Self) -> bool {
        other.next_item.key == self.next_item.key
    }
}
impl PartialOrd for HeapItem<SstableIterator> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            other
                .next_item
                .key
                .cmp(&self.next_item.key)
                .then_with(|| self.next_item.sequence.cmp(&other.next_item.sequence)),
        )
    }
}
pub struct Compaction {
    pub writer: SstableWriter,
}

impl Compaction {
    pub fn merge_iterators<'a>(
        iter_vec: Vec<SstableIterator>,
    ) -> impl Iterator<Item = SkipListKV<Vec<u8>, Vec<u8>>> {
        let mut heap = BinaryHeap::new();
        for mut iter in iter_vec {
            if let Some(next_item) = iter.next() {
                heap.push(HeapItem { next_item, iter });
            }
        }
        let mut last: Option<Vec<u8>> = None;
        iter::from_fn(move || {
            while let Some(top) = heap.pop() {
                let item = top.next_item;
                let mut iter = top.iter;
                if let Some(next_item) = iter.next() {
                    heap.push(HeapItem { next_item, iter });
                }
                if let Some(ref last_key) = last {
                    if last_key == &item.key {
                        continue;
                    }
                }
                last = Some(item.key.clone());
                return Some(item);
            }
            return None;
        })
    }

    pub fn compact(&mut self, iter_vec: Vec<SstableIterator>) -> Result<(), EngineError> {
        let merged = Self::merge_iterators(iter_vec);
        self.writer.write(merged)?;
        Ok(())
    }
}
