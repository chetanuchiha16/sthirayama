use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    path::Path,
    sync::atomic::Ordering,
};

use crate::{
    skiplist::SkipListKV,
    sstable::{
        GLOBAL_COUNT, data_block,
        errors::SsTableReaderError,
        footer::Footer,
        index::{BlockMeta, IndexBlock},
        manifest::Manifest,
    },
};

pub struct SstableReader<T: AsRef<Path>> {
    path: T,
    file: File,
}

impl<T: AsRef<Path>> SstableReader<T> {
    pub fn new(path: T) -> Result<Self, SsTableReaderError> {
        let file = OpenOptions::new()
            .read(true)
            // .append(true)
            .open(&path)?;
        Ok(Self { path, file })
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
    pub fn binary_search_index(
        &mut self,
        key: &Vec<u8>,
    ) -> Result<Option<usize>, SsTableReaderError> {

        let mut index_block = &self.read_index_block()?.blocks;
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

        let data_block_offset = block_meta.offset;
        let data_block_len = block_meta.len;

        self.file
            .seek(std::io::SeekFrom::Start(data_block_offset))?;
        let mut kv_list: Vec<SkipListKV<Vec<u8>, Vec<u8>>> = Vec::new();
        let mut i = 0;
        // println!("{:?}", block_meta);
        while i < data_block_len {
            let mut k_len_buffer = [0u8; 8];
            self.file.read_exact(&mut k_len_buffer)?;
            let k_len = usize::from_le_bytes(k_len_buffer);

            let mut k_bytes = vec![0u8; k_len];
            self.file.read_exact(&mut k_bytes)?;
            let k = str::from_utf8(&k_bytes)?;

            let mut v_len_bytes = [0u8; 8];
            self.file.read_exact(&mut v_len_bytes)?;
            let v_len = usize::from_le_bytes(v_len_bytes);

            let mut v_bytes = vec![0u8; v_len];
            self.file.read_exact(&mut v_bytes)?;
            let v = str::from_utf8(&v_bytes)?;

            let kv = SkipListKV::new(k_bytes.clone(), v_bytes.clone());

            kv_list.push(kv);
            i += k_len_buffer.len() + v_len_bytes.len() + k_bytes.len() + v_bytes.len();

            // let kv = &kv_list[0];
            // println!("{:?}", kv);
            // println!(
            //     "finding {} found {}",
            //     str::from_utf8(key)?,
            //     str::from_utf8(&kv.0)?
            // );
        }
        // println!("{:?}", kv_list);
        Ok(Some(kv_list))
    }

    
    #[cfg(not(feature = "use-legacy-search"))]
    pub fn binary_search_data(
        &mut self,
        key: &Vec<u8>,
    ) -> Result<Option<Vec<u8>>, SsTableReaderError> {
        let Some(data_block) = self.read_data_block(key)? else {
            return Ok(None);
        };
        
        let x = match data_block.binary_search_by_key(key, |data| data.0.clone()) {
            Ok(key_idx) => Some(data_block[key_idx].1.clone()),
            Err(_) => None,
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
