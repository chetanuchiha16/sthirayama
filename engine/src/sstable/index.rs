use std::{fs::File, io::Write};

use bitcode::{Decode, Encode};

use crate::sstable::errors::SsTableWriterError;

#[derive(Debug, Encode, Decode)]
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
    ///encode block meta into binary
    pub fn encode(&self) -> ([u8; 8], Vec<u8>) {
        let block_meta_bytes = bitcode::encode(self);
        let block_meta_bytes_len_as_bytes = block_meta_bytes.len().to_le_bytes();
        (block_meta_bytes_len_as_bytes, block_meta_bytes)
    }
}

#[derive(Debug, Encode, Decode)]
pub struct IndexBlock {
    pub blocks: Vec<BlockMeta>,
}

impl IndexBlock {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn encode(&self) -> ([u8; 8], Vec<u8>) {
        let bytes = bitcode::encode(self);
        let bytes_len = bytes.len().to_le_bytes();
        (bytes_len, bytes)
    }

    pub fn write_bytes_to(&self, file: &mut File) -> Result<(), SsTableWriterError> {
        let (bytes_len, bytes) = self.encode();
        file.write_all(&bytes_len)?;
        file.write_all(&bytes)?;
        Ok(())
    }
}
