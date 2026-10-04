use std::{
    fs::File,
    io::{Read, Seek, Write},
};

use crate::{
    skiplist::SkipListKV,
    sstable::{
        errors::{self, SsTableReaderError},
        index::BlockMeta,
    },
};

pub struct DataBlock {
    kv_list_bytes: Vec<u8>,
    pub size: usize,
    // pub last_key: Vec<u8>,
}

impl DataBlock {
    pub fn new() -> Self {
        Self {
            kv_list_bytes: Vec::new(),
            size: 0,
            // last_key: Vec::new(),
        }
    }

    // pub fn add(&mut self, len_byte: [u8; 8], data_byte: &Vec<u8>) {
    pub fn add(&mut self, key: &[u8], value: &[u8]) {
        let key_len_bytes = key.len().to_le_bytes();
        self.kv_list_bytes.extend_from_slice(&key_len_bytes);
        self.kv_list_bytes.extend_from_slice(key);

        let value_len_bytes = value.len().to_le_bytes();
        self.kv_list_bytes.extend_from_slice(&value_len_bytes);
        self.kv_list_bytes.extend_from_slice(value);

        self.size += key_len_bytes.len() + value_len_bytes.len() + key.len() + value.len();
    }

    pub fn can_fit(&self, entry_size: usize) -> bool {
        self.size + entry_size < 4000
    }
    
    pub fn read(
        file: &mut File,
        block_meta: &BlockMeta,
    ) -> Result<Option<Vec<SkipListKV<Vec<u8>, Vec<u8>>>>, SsTableReaderError> {
        let data_block_offset = block_meta.offset;
        let data_block_len = block_meta.len;

        file.seek(std::io::SeekFrom::Start(data_block_offset))?;
        let mut kv_list: Vec<SkipListKV<Vec<u8>, Vec<u8>>> = Vec::new();
        let mut i = 0;
        // println!("{:?}", block_meta);
        while i < data_block_len {
            let mut k_len_buffer = [0u8; 8];
            file.read_exact(&mut k_len_buffer)?;
            let k_len = usize::from_le_bytes(k_len_buffer);

            let mut k_bytes = vec![0u8; k_len];
            file.read_exact(&mut k_bytes)?;
            let _k = str::from_utf8(&k_bytes)?;

            let mut v_len_bytes = [0u8; 8];
            file.read_exact(&mut v_len_bytes)?;
            let v_len = usize::from_le_bytes(v_len_bytes);

            let mut v_bytes = vec![0u8; v_len];
            file.read_exact(&mut v_bytes)?;
            let _v = str::from_utf8(&v_bytes)?;

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
    pub fn write_to(&self, file: &mut impl Write) -> Result<(), errors::SsTableWriterError> {
        // let len = self.size.to_le_bytes();
        // file.write_all(&len);
        file.write_all(&self.kv_list_bytes)?;
        Ok(())
    }
}
