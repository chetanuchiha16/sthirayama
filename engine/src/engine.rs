use std::mem;

use crate::{
    memtable::Memtable,
    skiplist_error::SkipListError,
    sstable::{
        errors::{SsTableReaderError, SsTableWriterError},
        reader::SstableReader,
        writer::SstableWriter,
    },
    wal::Wal,
};

///Database Engine
pub struct Engine {
    memtable: Memtable,
    immutable_memtable: Option<Memtable>,
    wal: Wal,
    sstable_count: usize,
    // sstable : SstableWriter
}

impl Engine {
    pub fn new() -> Result<Self, SkipListError> {
        Ok(Self {
            memtable: Memtable::new(),
            immutable_memtable: None,
            wal: Wal::new()?,
            sstable_count: 0,
        })
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.wal.append(&key, &value);
        self.memtable.insert(key, value);
        let limit = 4 * 1024;
        if self.memtable.size > limit {
            println!("memtable size reached 4kb");
            self.flush();
        }
    }

    fn flush(&mut self) -> Result<(), SsTableWriterError> {
        let frozen = mem::replace(&mut self.memtable, Memtable::new());
        //because you need new sstable for a new frozen memtable anyway
        let mut sstable = SstableWriter::new(format!("{:06}.sst", self.sstable_count), frozen)?;
        sstable.write();
        // self.sstable_count += 1;
        Ok(())
    }

    pub fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, SsTableReaderError> {
        match self.memtable.skiplist.search(key.clone()) {
            Some(value) => {
                println!("from memtable");
                Ok(Some(value))
            }
            None => {
                let mut sstable = SstableReader::new(format!("{:06}.sst", self.sstable_count))?;
                println!("from sstable");
                sstable.binary_search_data(&key)
            }
        }
    }
}
