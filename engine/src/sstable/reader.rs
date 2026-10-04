use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek},
    path::Path,
};

#[cfg(not(feature = "use-legacy-search"))]
use crate::memtable::Value;
use crate::{
    config::get_sstable_path,
    skiplist::SkipListKV,
    sstable::{
        data_block::DataBlock, errors::SsTableReaderError, footer::Footer, index::IndexBlock,
        iterator::SstableIterator,
    },
};

pub struct SstableReader {
    // path: PathBuf,
    file: File,
}

impl SstableReader {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, SsTableReaderError> {
        // let sstable_root = PathBuf::from("../sstable");
        let sstable_path = get_sstable_path();
        let sstable_file = sstable_path?.join(path);
        let file = OpenOptions::new()
            .read(true)
            // .append(true)
            .open(&sstable_file)?;
        Ok(Self {
            // path: sstable_file,
            file,
        })
    }
    pub fn iter(&mut self) -> Result<SstableIterator<'_>, SsTableReaderError> {
        let index = self.read_index_block()?;
        let file = &mut self.file;
        Ok(SstableIterator {
            file,
            index,
            current_entry: 0,
            current_block: 0,
            // data_block:DataBlock::read(file, &index.blocks[0])?,
        })
    }
    pub fn read_footer(&mut self) -> Result<Footer, SsTableReaderError> {
        self.file.seek(std::io::SeekFrom::End(-8))?;
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;

        let len = usize::from_le_bytes(buf);
        let n: i64 = 8 + len as i64;
        self.file.seek(std::io::SeekFrom::End(-n))?;

        let mut buf = vec![0u8; len];
        self.file.read_exact(&mut buf)?;
        let ans: Footer = bitcode::decode(&buf)?;
        // println!("footer len read: {len}, footer read: {:?}", ans);
        Ok(ans)
    }

    pub fn read_index_block(&mut self) -> Result<IndexBlock, SsTableReaderError> {
        let footer = self.read_footer()?;
        let index_offset = footer.index_offset;
        // let index_len = footer.index_len as usize; // we can just get IndexBlock len instead of from footer

        self.file.seek(std::io::SeekFrom::Start(index_offset))?;

        let mut buffer = [0u8; 8];
        self.file.read_exact(&mut buffer)?;
        let index_len = usize::from_le_bytes(buffer);

        let mut buf = vec![0u8; index_len];
        self.file.read_exact(&mut buf)?;
        let index: IndexBlock = bitcode::decode(&buf)?;

        // println!("here {:?}", buf);
        // println!("index read: {:?}", index.blocks);
        Ok(index)
    }

    #[cfg(not(feature = "use-legacy-search"))]
    pub fn binary_search_index(&mut self, key: &[u8]) -> Result<Option<usize>, SsTableReaderError> {
        let index_block = &self.read_index_block()?.blocks;
        let idx = index_block.partition_point(|block_meta| block_meta.last_key.as_slice() < key);

        if idx < index_block.len() {
            Ok(Some(idx))
        } else {
            Ok(None)
        }
    }

    pub fn read_data_block(
        &mut self,
        key: &Vec<u8>,
    ) -> Result<Option<Vec<SkipListKV<Vec<u8>, Vec<u8>>>>, SsTableReaderError> {
        let Some(block_idx) = self.binary_search_index(key)? else {
            return Ok(None);
        };

        let index_block = self.read_index_block()?.blocks;
        let block_meta = &index_block[block_idx];
        let kv_list = DataBlock::read(&mut self.file, block_meta)?;
        Ok(Some(kv_list))
    }

    #[cfg(not(feature = "use-legacy-search"))]
    pub fn binary_search_data(&mut self, key: &Vec<u8>) -> Result<Value, SsTableReaderError> {
        let Some(data_block) = self.read_data_block(key)? else {
            return Ok(Value::None);
        };

        let x = match data_block.binary_search_by_key(key, |data| data.key.clone()) {
            Ok(key_idx) => {
                use crate::memtable::Value::{self};

                let x = data_block[key_idx].value.clone();
                Value::from_bytes(&x)?
                // match y {
                //     Value::Data(data) => Some(data),
                //     Value::Tombstone => None,
                //     Value::None => None,
                // }

                // Some(data_block[key_idx].value.clone())
            }
            Err(_) => Value::None,
        };

        Ok(x)
    }

    #[cfg(feature = "use-legacy-search")]
    pub fn binary_search_index(
        &mut self,
        key: &Vec<u8>,
    ) -> Result<Option<usize>, SsTableReaderError> {
        let mut index_block = self.read_index_block()?.blocks;
        let (mut left, mut right) = (0i32, index_block.len() as i32 - 1);
        let mut ans_idx: Option<usize> = None;

        while left <= right {
            let mid = left + (right - left) / 2;
            let mid_block = &index_block[mid as usize];
            let mid_block_key = &mid_block.last_key;
            if key <= mid_block_key {
                ans_idx = Some(mid as usize);
                right = mid - 1;
            } else {
                left = mid + 1;
            }
        }
        let key_val = str::from_utf8(key)?;
        let left_val = str::from_utf8(&index_block[0].last_key)?;
        if index_block.len() > 1 {
            let right_val = str::from_utf8(&index_block[1].last_key)?;
        }
        if let Some(answer_idx) = ans_idx {
            let found = str::from_utf8(&index_block[answer_idx as usize].last_key)?;

            // println!(
            //     "left: {}, key to find: {}, right: {}, found block's last key: {found}",
            //     left_val, key_val, right_val
            // );
        } else {
            // println!(
            //     "left: {}, key to find: {}, right: {}",
            //     left_val, key_val, right_val
            // );
            println!("{key_val} Not Found")
        }

        Ok(ans_idx)
    }

    #[cfg(feature = "use-legacy-search")]
    pub fn binary_search_data(
        &mut self,
        key: &Vec<u8>,
    ) -> Result<Option<Vec<u8>>, SsTableReaderError> {
        let Some(data_block) = self.read_data_block(key)? else {
            return Ok(None);
        };
        let (mut left, mut right) = (0, data_block.len() as i32 - 1);

        while left <= right {
            let mid = left + (right - left) / 2;
            let mid_val = &data_block[mid as usize];
            if mid_val.0 == *key {
                return Ok(Some(mid_val.1.clone()));
            } else if *key < mid_val.0 {
                right = mid - 1;
            } else {
                left = mid + 1;
            }
        }

        Ok(None)
    }
}
