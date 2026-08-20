use std::{
    fs::create_dir_all,
    mem,
    path::{Path, PathBuf},
};

use crate::{
    config::get_sstable_path,
    engine_error::EngineError,
    memtable::Memtable,
    sstable::{reader::SstableReader, writer::SstableWriter},
    wal::Wal,
};
struct SstableMeta {
    sstable_no: usize,
    last_key: Vec<u8>,
}
impl SstableMeta {
    pub fn new(sstable_no: usize, last_key: &Vec<u8>) -> Self {
        Self {
            sstable_no,
            last_key: last_key.clone(),
        }
    }
}

///Database Engine
pub struct Engine {
    memtable: Memtable,
    // immutable_memtable: Option<Memtable>,
    wal: Wal,
    sstable_count: usize,
    // ssts: File,
    // sstable : SstableWriter
    path: PathBuf,
    sstable_meta_list: Vec<SstableMeta>,
}

impl Engine {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, EngineError> {
        let path = get_sstable_path()?.join(&path);
        create_dir_all(&path)?;
        // let path = path.as_ref().join(".sst");
        // let ssts_path = get_sstable_path().join(&path);

        // let mut file = OpenOptions::new()
        //     .read(true)
        //     .write(true)
        //     .append(true)
        //     .create(true)
        //     .open(ssts_path)?;
        Ok(Self {
            memtable: Memtable::new(),
            // immutable_memtable: None,
            wal: Wal::new()?,
            sstable_count: 0,
            // ssts: file,
            path: path,
            sstable_meta_list: Vec::new(),
        })
    }

    pub fn set(&mut self, key: &Vec<u8>, value: Vec<u8>) -> Result<(), EngineError> {
        self.wal.append(key, &value)?;
        self.memtable.insert(key, value)?;
        let limit = 4 * 1024;
        if self.memtable.size > limit {
            // println!("memtable size reached 4kb");
            self.flush()?;
            // self.ssts.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), EngineError> {
        let frozen = mem::replace(&mut self.memtable, Memtable::new());
        //because you need new sstable for a new frozen memtable anyway
        let path = self.path.join(format!("{:06}.sst", self.sstable_count));
        let mut sstable = SstableWriter::new(path, frozen)?;

        // let sst_count_buf = self.sstable_count.to_le_bytes();
        // self.ssts.write_all(&sst_count_buf);

        let last_key = sstable.write()?;
        // let last_key_len = last_key.len();
        // self.ssts.write_all(&last_key_len.to_le_bytes())?;
        // self.ssts.write_all(&last_key)?;
        // self.ssts.flush()?;
        // println!("flusing");
        let sstable_meta = SstableMeta::new(self.sstable_count, &last_key);
        self.sstable_meta_list.push(sstable_meta);
        self.sstable_count += 1;
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

    fn get_key_file_path(&mut self, key: &Vec<u8>) -> Result<Option<usize>, EngineError> {
        // let last_keys_count = self.get_count_last_key()?;
        let last_keys_count = &self.sstable_meta_list;
        let idx = last_keys_count.partition_point(|meta| meta.last_key.as_slice() < key);
        if idx < last_keys_count.len() {
            // let key = last_keys_count[idx].clone();
            // let val = count_last_key.get(&key).cloned();
            // let (key, val) = &last_keys_count[idx];
            let x = &last_keys_count[idx];
            Ok(Some(x.sstable_no))
        } else {
            Ok(None)
        }
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, EngineError> {
        match self.memtable.extract(key)? {
            Some(value) => {
                println!("from memtable");
                Ok(Some(value))
            }
            None => {
                let Some(sstable_no) = self.get_key_file_path(&key)? else {
                    return Ok(None);
                };
                let path = self.path.join(format!("{:06}.sst", sstable_no));
                let mut sstable = SstableReader::new(path)?;
                println!("from sstable {}", sstable_no);

                Ok(sstable.binary_search_data(&key)?)
            }
        }
    }
}
