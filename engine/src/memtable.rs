use crate::{
    skiplist::SkipList,
    skiplist_error::SkipListError,
    sstable::{errors::SsTableWriterError, writer::SstableWriter},
};

pub struct Memtable {
    pub skiplist: SkipList<Vec<u8>, Vec<u8>>,
    pub size: usize,
}

impl Memtable {
    pub fn new() -> Self {
        Self {
            skiplist: SkipList::new(5, b"0".to_vec(), b"0".to_vec()),
            size: 0,
        }
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.size += key.len() + value.len();
        // println!("{}", self.size);
        self.skiplist.insert(key, value);
    }
}
