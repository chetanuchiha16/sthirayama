pub struct Footer {
    index_offset: u64,
    index_len: u64,
}

impl Footer {
    pub fn new(index_offset: u64, index_len: u64) -> Self {
        Self {
            index_offset,
            index_len
        }
    }

    pub fn encode(&self) -> ([u8; 8], Vec<u8>) {
        let index_offset_byte = bitcode::encode(&self.index_offset);
        let index_offset_byte_len_byte = self.index_offset.to_le_bytes();
        (index_offset_byte_len_byte, index_offset_byte)
    }
}
