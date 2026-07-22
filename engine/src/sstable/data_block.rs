use crate::skiplist::SkipListKV;

pub struct DataBlock {
    kv_list_bytes: Vec<u8>,
}

impl DataBlock {
    pub fn new() -> Self {
        Self {
            kv_list_bytes: Vec::new(),
        }
    }

    pub fn add(&mut self, kv: SkipListKV<Vec<u8>, Vec<u8>>) {
        let (len_byte, data_byte) = kv.encode(); // put all bytes from both
        self.kv_list_bytes.extend_from_slice(&len_byte);
        self.kv_list_bytes.extend_from_slice(&data_byte);
    }
}
