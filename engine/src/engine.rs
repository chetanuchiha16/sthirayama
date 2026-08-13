use std::mem;

use crate::{
    memtable::Memtable, skiplist_error::SkipListError, sstable::{errors::SsTableWriterError, writer::SstableWriter}, wal::Wal,
};

pub struct Engine {
    memtable: Memtable,
    immutable_memtable: Option<Memtable>,
    wal: Wal,
    // sstable : SstableWriter
}

impl Engine {
    pub fn new() -> Result<Self, SkipListError> {
        Ok(Self {
            memtable: Memtable::new(),
            immutable_memtable: None,
            wal: Wal::new()?,
        })
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.wal.append(&key, &value);
        self.memtable.insert(key, value);
        if self.memtable.size > 4 * 1024 * 1024 {
            self.flush();
        }
    }

    fn flush(&mut self) -> Result<(), SsTableWriterError>{
        let frozen = mem::replace(&mut self.memtable, Memtable::new());
        //because you need new sstable for a new frozen memtable anyway
        let mut sstable = SstableWriter::new("sstable.sst", frozen)?;
        sstable.write();
        Ok(())
    }
}
