use std::{
    collections::binary_heap,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
};

use crate::{
    skiplist::{self, SkipList, SkipListKV, SkipListNode},
    sstable::{
        data_block::DataBlock, errors::SsTableWriterError, footer::Footer, index::BlockMeta,
    },
};

pub struct SstableWriter {
    file: File,
    skiplist: SkipList<Vec<u8>, Vec<u8>>,
    index: Vec<BlockMeta>,
}

impl SstableWriter {
    pub fn new(skiplist: SkipList<Vec<u8>, Vec<u8>>) -> Result<Self, SsTableWriterError> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open("table.sst")?;
        let index: Vec<BlockMeta> = Vec::new();
        Ok(Self {
            file,
            skiplist,
            index,
        })
    }

    pub fn write(&mut self) -> Result<(), SsTableWriterError> {
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
        //         self.index.push(block);
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
        let mut offset = self.file.stream_position()?;
        let mut data_block = DataBlock::new();
        let mut last_key = &Vec::new();
        for kv in self.skiplist.iter() {
            let (len_byte, data_byte) = kv.encode();
            let entry_size = len_byte.len() + data_byte.len();
            
            if !data_block.can_fit(entry_size) {
                let block_meta = BlockMeta::new(data_block.size, offset, last_key.to_vec());
                println!("{:?}", str::from_utf8(&block_meta.last_key));
                offset = self.file.stream_position()?;
                self.index.push(block_meta);
                println!("{:?}", self.index);
                data_block = DataBlock::new();
            }
            
            data_block.add(len_byte, &data_byte);
            last_key = &kv.key;

            self.file.write_all(&len_byte);
            self.file.write_all(&data_byte);
        }

        let index_offset = self.file.stream_position()?;

        /// writing blockMeta/index block
        for block in self.index.iter() {
            let (block_meta_bytes_len_as_bytes, block_meta_bytes) = block.encode();
            self.file.write_all(&block_meta_bytes_len_as_bytes);
            self.file.write_all(&block_meta_bytes);
        }
        
        ///writing footer
        let index_len = self.file.stream_position()? - index_offset;
        let footer = Footer::new(index_offset, index_len);
        let (index_offset_len, index_offset_byte) = footer.encode();
        self.file.write_all(&index_offset_len);
        self.file.write_all(&index_offset_byte);
        
        
        self.file.flush();
        Ok(())
    }

    // to verify for now, maybe moved later
    pub fn read(&mut self) -> Result<(), SsTableWriterError> {
        self.file.seek(io::SeekFrom::Start(0));
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;
        let data_len = usize::from_le_bytes(buf);

        let mut buf = vec![0u8; data_len];
        self.file.read_exact(&mut buf);
        let data: SkipListKV<Vec<u8>, Vec<u8>> = bitcode::decode(&buf)?;
        println!("{:?}", data);
        Ok(())
    }
}
