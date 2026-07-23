use bitcode::Encode;

#[derive(Debug, Encode)]
pub struct BlockMeta {
    len: usize,
    offset: u64,
    pub last_key: Vec<u8>,
}

impl BlockMeta {
    pub fn new(len: usize, offset: u64, last_key: Vec<u8>) -> Self {
        Self {
            len,
            offset,
            last_key,
        }
    }

    pub fn encode(&self) -> ([u8; 8], Vec<u8>) {
        let block_meta_bytes = bitcode::encode(self);
        let block_meta_bytes_len_as_bytes = block_meta_bytes.len().to_le_bytes();
        (block_meta_bytes_len_as_bytes, block_meta_bytes)
    }
}
