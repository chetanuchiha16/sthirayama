use std::{
    borrow::Borrow,
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    path::Path,
};

use crate::{
    config::get_sstable_path,
    skiplist::SkipListKV,
    sstable::{
        data_block::DataBlock,
        errors::SsTableWriterError,
        footer::Footer,
        index::{BlockMeta, IndexBlock},
    },
};

pub struct SstableWriter {
    file: File,
    // memtable: Memtable,
    index: IndexBlock,
    bytes: Vec<u8>,
}

impl SstableWriter {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, SsTableWriterError> {
        let sstable_path = get_sstable_path();

        let sstable_file = sstable_path?.join(path);
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&sstable_file)?;
        let index = IndexBlock::new();
        Ok(Self {
            file,
            // memtable,
            index,
            bytes: Vec::new(),
        })
    }
    pub fn build<I, T>(&mut self, memtable: I) -> Result<Vec<u8>, SsTableWriterError>
    where
        I: Iterator<Item = T>,
        T: Borrow<SkipListKV<Vec<u8>, Vec<u8>>>,
    {
        // writing data block
        let mut data_block = DataBlock::new();
        let mut last_key = Vec::new();
        let mut offset = 0;
        for kv in memtable {
            let key = &kv.borrow().key;
            let value = &kv.borrow().value;
            let sequence = &kv.borrow().value;
            let key_len_bytes = key.len().to_le_bytes();
            let value_len_bytes = value.len().to_le_bytes();

            let seq_len_bytes = sequence.len().to_le_bytes();
            let entry_size = key_len_bytes.len()
                + value_len_bytes.len()
                + key.len()
                + value.len()
                + seq_len_bytes.len()
                + sequence.len();

            if !data_block.can_fit(entry_size) {
                let block_meta = BlockMeta::new(data_block.size, offset, last_key.to_vec());
                self.index.blocks.push(block_meta);
                let initial = self.bytes.len() as u64;
                data_block.write_to(&mut self.bytes)?;
                offset += self.bytes.len() as u64 - initial;
                data_block = DataBlock::new();
            }

            data_block.add(key.as_slice(), value.as_slice(), sequence.as_slice());
            last_key = key.to_vec();
        }

        if data_block.size > 0 {
            let block_meta = BlockMeta::new(data_block.size, offset, last_key.to_vec());
            self.index.blocks.push(block_meta);
            let initial = self.bytes.len() as u64;
            data_block.write_to(&mut self.bytes)?;
            offset += self.bytes.len() as u64 - initial;
        }

        let index_offset = offset;

        // writing blockMeta/index block
        self.index.write_bytes_to(&mut self.bytes)?;

        //writing footer
        let index_len = self.bytes.len() as u64 - index_offset;
        let footer = Footer::new(index_offset, index_len);
        footer.write_to(&mut self.bytes)?;

        Ok(last_key.to_owned())
    }

    pub fn write<I, T>(&mut self, memtable: I) -> Result<Vec<u8>, SsTableWriterError>
    where
        I: Iterator<Item = T>,
        T: Borrow<SkipListKV<Vec<u8>, Vec<u8>>>,
    {
        let last_key = self.build(memtable)?;
        self.file.seek(io::SeekFrom::Start(0))?;
        self.file.write_all(&self.bytes)?;
        self.file.flush()?;
        Ok(last_key)
    }
}
