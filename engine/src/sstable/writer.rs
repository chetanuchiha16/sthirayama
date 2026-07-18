use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
};

use crate::{
    skiplist::{self, SkipList, SkipListKV, SkipListNode},
    sstable::index::BlockMeta,
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
        let head = &self.skiplist.head.unwrap();
        let mut current = SkipListNode::get_forward(head)[0];
        let mut size = 0usize;
        let mut offset = 0usize;
        while let Some(cur_node) = current {
            let data = SkipListNode::get_data(&cur_node).clone();
            let mut last_key = data.key.clone();
            let encoded_data = bitcode::encode(&data);
            let data_len = encoded_data.len();
            let encoded_data_len = data_len.to_le_bytes();
            size += data_len + encoded_data_len.len();
            println!("{}", size);
            if size > 4000 {
                let block = BlockMeta::new(size, offset, last_key);
                self.blocks.push(block);
                println!("{:?}", self.blocks);
                offset = size;
                size = 0;
            }
            self.file.write_all(&encoded_data_len);
            self.file.write_all(&encoded_data);
            self.file.flush();
            println!(
                "{} : {}",
                String::from_utf8(data.key).unwrap(),
                String::from_utf8(data.value).unwrap()
            );
            let next_node = SkipListNode::get_forward(&cur_node)[0];
            current = next_node;
        }

        for block in self.blocks.iter() {
            let (mut block_meta_bytes_len_as_bytes, mut block_meta_bytes) = block.encode();
            self.file.write_all(&mut block_meta_bytes_len_as_bytes);
            self.file.write_all(&mut block_meta_bytes);
        }
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
