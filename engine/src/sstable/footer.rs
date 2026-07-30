#[derive(Debug)]
pub struct Footer {
    index_offset: u64,
    index_len: u64,
}

impl Footer {
    pub fn new(index_offset: u64, index_len: u64) -> Self {
        println!(
            "index len inside new {} and index offset is {}",
            index_len, index_offset
        );
        Self {
            index_offset,
            index_len,
        }
    }

    pub fn encode(&self) -> ([u8; 8], Vec<u8>) {
        let index_offset_byte = bitcode::encode(&self.index_offset);
        let index_offset_byte_len_byte = index_offset_byte.len().to_le_bytes();
        // println!("ioblb {}", usize::from_le_bytes(index_offset_byte_len_byte));

        (index_offset_byte_len_byte, index_offset_byte)
    }
}
