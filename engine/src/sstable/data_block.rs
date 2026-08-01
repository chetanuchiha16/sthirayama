use std::{fs::File, io::Write};

use crate::skiplist::SkipListKV;

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

    pub fn write_to(&self, file: &mut File) {
        // let len = self.size.to_le_bytes();
        // file.write_all(&len);
        file.write_all(&self.kv_list_bytes);
    }
}
