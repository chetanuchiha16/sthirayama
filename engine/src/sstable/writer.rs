use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
};

use crate::{
    skiplist::{self, SkipList, SkipListKV, SkipListNode},
    sstable::{data_block::DataBlock, index::BlockMeta},
};

pub struct SstableWriter {
    file: File,
    skiplist: SkipList<Vec<u8>, Vec<u8>>,
    blocks: Vec<BlockMeta>,
}

impl SstableWriter {
    pub fn new(skiplist: SkipList<Vec<u8>, Vec<u8>>) -> Result<Self, io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open("table.sst")?;
        let blocks: Vec<BlockMeta> = Vec::new();
        Ok(Self {
            file,
            skiplist,
            blocks,
        })
    }

    pub fn write(&mut self) {
        self.file.seek(io::SeekFrom::Start(0));

        /// writing data block
        // let mut size = 0usize;
        // let mut offset = 0usize;
        // ver 1 encode print, encode print, when 4kb create new block meta
        // for kv in self.skiplist.iter() {
        //     let last_key = &kv.key;
        //     let (encoded_data_len, encoded_data) = kv.encode();

        //     size += encoded_data.len() + encoded_data_len.len();
        //     println!("{}", size);

        //     if size > 4000 {
        //         let block = BlockMeta::new(size, offset, last_key.clone());
        //         self.blocks.push(block);
        //         offset = size;
        //         size = 0;
        //     }

        //     // println!(
        //         //     "{} : {}",
        //         //     String::from_utf8(kv.key).unwrap(),
        //         //     String::from_utf8(kv.value).unwrap()
        //         // );
        // }

        //ver 2 build upto 4kb print
        let mut size = 0usize;
        let mut offset = self.file.stream_position().unwrap();
        let mut data_block = DataBlock::new();
        let mut last_key = Vec::new();
        for kv in self.skiplist.iter() {
            let (len_byte, data_byte) = kv.encode();
            let entry_size = len_byte.len() + data_byte.len();

            if (!data_block.can_fit(entry_size)) {
                let block_meta = BlockMeta::new(data_block.size, offset, last_key.clone());
                offset = self.file.stream_position().unwrap();
                self.blocks.push(block_meta);
                println!("{:?}", self.blocks);
                data_block = DataBlock::new();
            }

            data_block.add(len_byte, &data_byte);
            last_key = kv.key.clone();

            self.file.write_all(&len_byte);
            self.file.write_all(&data_byte);
        }

        /// writing blockMeta/index block
        for block in self.blocks.iter() {
            let (block_meta_bytes_len_as_bytes, block_meta_bytes) = block.encode();
            self.file.write_all(&block_meta_bytes_len_as_bytes);
            self.file.write_all(&block_meta_bytes);
        }
        self.file.flush();
    }

    // to verify for now, maybe moved later
    pub fn read(&mut self) {
        self.file.seek(io::SeekFrom::Start(0));
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf).unwrap();
        let data_len = usize::from_le_bytes(buf);

        let mut buf = vec![0u8; data_len];
        self.file.read_exact(&mut buf);
        let data: SkipListKV<Vec<u8>, Vec<u8>> = bitcode::decode(&buf).unwrap();
        println!("{:?}", data)
    }
}
