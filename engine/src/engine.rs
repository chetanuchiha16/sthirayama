use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    mem,
    path::PathBuf,
};

use crate::{
    config::get_sstable_path,
    engine_error::EngineError,
    memtable::Memtable,
    skiplist_error::SkipListError,
    sstable::{
        self,
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
    ssts: File, // sstable : SstableWriter
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        let ssts_path = get_sstable_path().join("ssts.sst");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(ssts_path)?;
        Ok(Self {
            memtable: Memtable::new(),
            immutable_memtable: None,
            wal: Wal::new()?,
            sstable_count: 0,
            ssts: file,
        })
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.wal.append(&key, &value);
        self.memtable.insert(key, value);
        let limit = 4 * 1024;
        if self.memtable.size > limit {
            // println!("memtable size reached 4kb");
            self.flush();
            self.ssts.flush();
        }
    }

    fn flush(&mut self) -> Result<(), EngineError> {
        let frozen = mem::replace(&mut self.memtable, Memtable::new());
        //because you need new sstable for a new frozen memtable anyway
        let mut sstable = SstableWriter::new(format!("{:06}.sst", self.sstable_count), frozen)?;

        let sst_count_buf = self.sstable_count.to_le_bytes();
        self.ssts.write_all(&sst_count_buf);

        let last_key = sstable.write()?;
        let last_key_len = last_key.len();
        self.ssts.write_all(&last_key_len.to_le_bytes())?;
        self.ssts.write_all(&last_key)?;
        self.sstable_count += 1;
        Ok(())
    }

    fn get_count_last_key(&mut self) -> Result<Vec<(Vec<u8>, usize)>, EngineError> {
        // let mut count_last_key: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut last_keys_count: Vec<(Vec<u8>, usize)> = Vec::new();
        self.ssts.seek(std::io::SeekFrom::Start(0));
        for i in (0..self.sstable_count) {
            let mut buf = [0u8; 8];
            self.ssts.read_exact(&mut buf)?;
            let count = usize::from_le_bytes(buf);
            let mut last_key_len_buf = [0u8; 8];
            self.ssts.read_exact(&mut last_key_len_buf)?;
            let mut last_key_buf = vec![0u8; usize::from_le_bytes(last_key_len_buf)];
            self.ssts.read_exact(&mut last_key_buf)?;
            // let last_key = str::from_utf8(&last_key_buf)?;
            // println!("{:?}", count);
            // count_last_key.insert(last_key_buf.clone(), count);
            println!(
                "last key {:?}, count {:?}",
                &str::from_utf8(&last_key_buf)?,
                count
            );
            last_keys_count.push((last_key_buf, count));
        }
        Ok(last_keys_count)
    }

    fn get_key_file_path(&mut self, key: &Vec<u8>) -> Result<Option<usize>, EngineError> {
        let last_keys_count = self.get_count_last_key()?;
        let idx = last_keys_count.partition_point(|(last_key, _)| last_key < key);
        if idx < last_keys_count.len() {
            // let key = last_keys_count[idx].clone();
            // let val = count_last_key.get(&key).cloned();
            let (key, val) = &last_keys_count[idx];
            Ok(Some(*val))
        } else {
            Ok(None)
        }
    }

    pub fn get(&mut self, key: Vec<u8>) -> Result<Option<Vec<u8>>, EngineError> {
        match self.memtable.skiplist.search(key.clone()) {
            Some(value) => {
                println!("from memtable");
                Ok(Some(value))
            }
            None => {
                let Some(sstable_no) = self.get_key_file_path(&key)? else {
                    return Ok(None);
                };
                let mut sstable = SstableReader::new(format!("{:06}.sst", sstable_no))?;
                println!("from sstable {}", sstable_no);
                Ok(sstable.binary_search_data(&key)?)
            }
        }
    }
}
