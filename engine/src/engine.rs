use std::{
    fs::{self, create_dir_all, remove_file},
    mem,
    path::{Path, PathBuf},
    str::from_utf8,
};

use crate::{
    compaction::Compaction,
    config::get_sstable_path,
    engine_error::EngineError,
    memtable::{
        Memtable,
        Value::{self, Data, Tombstone},
    },
    sstable::{iterator::SstableIterator, reader::SstableReader, writer::SstableWriter},
    wal::Wal,
};

// struct SstableMeta {
//     sstable_no: usize,
//     last_key: Vec<u8>,
// }

// impl SstableMeta {
//     pub fn new(sstable_no: usize, last_key: &Vec<u8>) -> Self {
//         Self {
//             sstable_no,
//             last_key: last_key.clone(),
//         }
//     }
// }

///Database Engine
pub struct Engine {
    memtable: Memtable,
    // immutable_memtable: Option<Memtable>,
    wal: Wal,
    sstable_count: usize,
    active_sstables: Vec<usize>,
    // ssts: File,
    // sstable : SstableWriter
    path: PathBuf,
    // sstable_meta_list: Vec<SstableMeta>,
    sequence: usize,
}

impl Engine {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, EngineError> {
        let path = get_sstable_path()?.join(&path); //sstable/main
        create_dir_all(&path)?;
        // let path = path.as_ref().join(".sst");
        // let ssts_path = get_sstable_path().join(&path);

        // let mut file = OpenOptions::new()
        //     .read(true)
        //     .write(true)
        //     .append(true)
        //     .create(true)
        //     .open(ssts_path)?;
        let mut sstable_count = 0usize;
        let mut active_sstables = Vec::new();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "sst" {
                    if let Some(file_name) = entry.path().file_stem() {
                        if let Some(file_name_str) = file_name.to_str() {
                            let num: usize = file_name_str.parse()?;
                            sstable_count = sstable_count.max(num + 1);
                            active_sstables.push(num);
                        }
                    }
                }
            }
        }
        active_sstables.sort();
        let mut memtable = Memtable::new();
        let mut wal = Wal::new(path.join("wal"))?;
        wal.recover::<Vec<u8>, Vec<u8>>(&mut memtable.skiplist)?;
        Ok(Self {
            memtable,
            // immutable_memtable: None,
            wal,
            sstable_count,
            active_sstables,
            // ssts: file,
            path: path,
            // sstable_meta_list: Vec::new(),
            sequence: 0,
        })
    }

    pub fn run(&self) -> Result<(), EngineError> {
        let sstable = SstableReader::new(self.path.clone(), 11)?;
        for kv in sstable.iter()? {
            let key = kv.key;
            let value = Value::from_bytes(&kv.value).unwrap();
            let seq = usize::from_le_bytes(kv.sequence.try_into().unwrap());
            println!(
                "{} -> {:?} sequence {}",
                from_utf8(&key).unwrap(),
                value,
                seq
            )
        }
        Ok(())
    }

    pub fn set(&mut self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), EngineError> {
        // let start = Instant::now();
        self.sequence += 1;
        self.wal.append(key, Data(value.to_vec()), self.sequence)?;
        self.memtable.insert(key, value.clone(), self.sequence)?;
        let limit = 4 * 1024;

        if self.memtable.size > limit {
            // println!("memtable size reached 4kb");
            let frozen = mem::replace(&mut self.memtable, Memtable::new());
            //because you need new sstable for a new frozen memtable anyway
            let sstable_no = self.sstable_count;
            self.active_sstables.push(sstable_no);
            self.sstable_count += 1;
            let (old_wal, archived_path) = self.wal.rotate()?;
            // tokio::task::spawn_blocking(move || {
            Self::flush(frozen, self.path.clone(), sstable_no)?;
            if self.active_sstables.len() > 4 {
                self.compact()?;
            }
            // println!("flushed");
            Wal::recycle(old_wal, archived_path)?;
            // println!("recycled");
            // Ok::<(), EngineError>(())
            // });
            // self.ssts.flush()?;
        }
        // println!(
        //     "set {} in {:?}",
        //     str::from_utf8(&key)?,
        //     Instant::now() - start
        // );
        Ok(())
    }

    fn flush(frozen: Memtable, path: PathBuf, file_no: usize) -> Result<(), EngineError> {
        // tokio::task::spawn(async move {
        let mut sstable = SstableWriter::new(path, file_no)?;

        // let sst_count_buf = self.sstable_count.to_le_bytes();
        // self.ssts.write_all(&sst_count_buf);

        let _ = sstable.write(frozen.skiplist.iter())?;
        // let _ = sstable.write()?;
        // let last_key_len = last_key.len();
        // self.ssts.write_all(&last_key_len.to_le_bytes())?;
        // self.ssts.write_all(&last_key)?;
        // self.ssts.flush()?;
        // println!("flusing");
        // let sstable_meta = SstableMeta::new(self.sstable_count, &last_key);
        // self.sstable_meta_list.push(sstable_meta);
        //     Ok::<(), EngineError>(())
        // });
        Ok(())
    }

    pub fn compact(&mut self) -> Result<(), EngineError> {
        if self.active_sstables.len() <= 4 {
            return Ok(());
        }
        let to_compact = self.active_sstables[..4].to_vec();
        let sstable_no = self.sstable_count;
        // self.active_sstables.push(sstable_no);
        self.sstable_count += 1;
        let sstable = SstableWriter::new(self.path.clone(), sstable_no)?;
        let sstable_iters = self.get_sst_iters(&to_compact)?;
        let mut comp = Compaction { writer: sstable };
        comp.compact(sstable_iters)?;
        for &i in to_compact.iter() {
            let path = self.path.join(format!("{:06}.sst", i));
            remove_file(path)?;
        }
        // self.active_sstables = self.active_sstables[4..].to_vec();
        self.active_sstables.drain(..4);
        self.active_sstables.insert(0, sstable_no);
        Ok(())
    }

    fn get_sst_iters(&self, to_compact: &[usize]) -> Result<Vec<SstableIterator>, EngineError> {
        let mut vec = Vec::new();
        for &i in to_compact.into_iter() {
            let sstable = SstableReader::new(self.path.clone(), i)?;
            let iterator = sstable.iter()?;
            vec.push(iterator);
        }
        Ok(vec)
    }

    pub fn print_iters(&mut self, to_compact: &[usize]) -> Result<(), EngineError> {
        let iter_vec = self.get_sst_iters(to_compact)?;
        let merged = Compaction::merge_iterators(iter_vec);
        for m in merged {
            println!("{}", str::from_utf8(&m.key)?);
        }
        Ok(())
    }

    // fn get_count_last_key(&mut self, i: usize) -> Result<Vec<(Vec<u8>, usize)>, EngineError> {
    //     // let mut count_last_key: HashMap<Vec<u8>, usize> = HashMap::new();
    //     self.ssts.seek(std::io::SeekFrom::Start(0))?;
    //     let mut last_keys_count: Vec<(Vec<u8>, usize)> = Vec::new();
    //     // for i in (0..self.sstable_count) {
    //         let mut buf = [0u8; 8];
    //         println!("last key should print after this");
    //         self.ssts.read_exact(&mut buf)?;
    //         let count = usize::from_le_bytes(buf);
    //         let mut last_key_len_buf = [0u8; 8];
    //         self.ssts.read_exact(&mut last_key_len_buf)?;
    //         let mut last_key_buf = vec![0u8; usize::from_le_bytes(last_key_len_buf)];
    //         self.ssts.read_exact(&mut last_key_buf)?;
    //         // let last_key = str::from_utf8(&last_key_buf)?;
    //         // println!("{:?}", count);
    //         // count_last_key.insert(last_key_buf.clone(), count);
    //         println!(
    //             "last key {:?}, count {:?}",
    //             &str::from_utf8(&last_key_buf)?,
    //             count
    //         );
    //         last_keys_count.push((last_key_buf, count));
    //     // }
    //     Ok(last_keys_count)
    // }

    // fn get_key_file_path(&mut self, key: &Vec<u8>) -> Result<Option<usize>, EngineError> {
    //     // let last_keys_count = self.get_count_last_key()?;
    //     let last_keys_count = &self.sstable_meta_list;
    //     let idx = last_keys_count.partition_point(|meta| meta.last_key.as_slice() < key);
    //     if idx < last_keys_count.len() {
    //         // let key = last_keys_count[idx].clone();
    //         // let val = count_last_key.get(&key).cloned();
    //         // let (key, val) = &last_keys_count[idx];
    //         let x = &last_keys_count[idx];
    //         Ok(Some(x.sstable_no))
    //     } else {
    //         Ok(None)
    //     }
    // }

    pub fn get(&mut self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, EngineError> {
        match self.memtable.extract(key)? {
            Value::Data(data) => {
                // println!("from memtable");
                return Ok(Some(data));
            }
            Value::Tombstone => return Ok(None),
            Value::None => {
                // let Some(i) = self.get_key_file_path(&key)? else {
                //     return Ok(None);
                // };
                for &i in self.active_sstables.iter().rev() {
                    let mut sstable = SstableReader::new(self.path.clone(), i)?;

                    // if let Some(res) = sstable.binary_search_data(&key)? {
                    //     println!("from sstable {}", i);
                    //     return Ok(Some(res));
                    // }
                    match sstable.binary_search_data(&key)? {
                        Value::Data(val) => {
                            // println!("from sstable {}", i);
                            return Ok(Some(val));
                        }
                        Value::Tombstone => return Ok(None),
                        Value::None => continue,
                    }
                }

                Ok(None)
            } // Some(value) => {
              //     match value {
              //         Value::Tombstone => return Ok(None),
              //         Value::Data(data) => return Ok(Some(data)),
              //         Value::None => return Ok(None)
              //     }
              //     // return Ok(Some(value))
              // }
              // None => {
              //     let Some(i) = self.get_key_file_path(&key)? else {
              //         return Ok(None);
              //     };
              //     // let mut res = None;
              //     // for i in (0..self.sstable_count).rev() {
              //     let path = self.path.join(format!("{:06}.sst", i));
              //     let mut sstable = SstableReader::new(path)?;

              //     if let Some(res) = sstable.binary_search_data(&key)? {
              //         println!("from sstable {}", i);
              //         return Ok(Some(res));
              //     }
              //     // }

              //     Ok(None)
              // }
        }
    }
    ///delete key
    pub fn del(&mut self, key: &Vec<u8>) -> Result<(), EngineError> {
        self.sequence += 1;
        self.wal.append(key, Tombstone, self.sequence)?;
        println!("delete {}", str::from_utf8(key).unwrap());
        self.memtable.delete(key, self.sequence);
        Ok(())
    }
}
