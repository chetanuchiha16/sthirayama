use std::{
    collections::binary_heap,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use crate::{
    memtable::{self, Memtable}, skiplist::{self, SkipList, SkipListKV, SkipListNode}, sstable::{
        GLOBAL_COUNT,
        data_block::DataBlock,
        errors::SsTableWriterError,
        footer::Footer,
        index::{BlockMeta, IndexBlock},
        manifest::Manifest,
    },
};

pub struct SstableWriter {
    path: PathBuf,
    file: File,
    memtable: Memtable,
    // skiplist: SkipList<Vec<u8>, Vec<u8>>,
    index: IndexBlock,
}

impl SstableWriter {
    pub fn new<T: AsRef<Path>>(
        path: T,
        // skiplist: SkipList<Vec<u8>, Vec<u8>>,
        memtable: Memtable
    ) -> Result<Self, SsTableWriterError> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            // .append(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        let index = IndexBlock::new();
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            file,
            // skiplist,
            memtable,
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
        let mut data_block = DataBlock::new();
        let mut last_key = &Vec::new();
        for SkipListKV(key, value) in self.memtable.skiplist.iter() {
            // if key == b"99" {
            //     println!("found 99")
            // }
            // let (len_byte, data_byte) = kv.encode();
            let key_len_bytes = key.len().to_le_bytes();
            let value_len_bytes = value.len().to_le_bytes();

            let entry_size = key_len_bytes.len() + value_len_bytes.len() + key.len() + value.len();

            if !data_block.can_fit(entry_size) {
                let offset = self.file.stream_position()?;
                let block_meta = BlockMeta::new(data_block.size, offset, last_key.to_vec());
                // println!("{:?}", str::from_utf8(&block_meta.last_key));
                self.index.blocks.push(block_meta);
                // println!("index written: {:?}", self.index.blocks);
                data_block.write_to(&mut self.file);
                data_block = DataBlock::new();
            }

            data_block.add(key, value);
            last_key = &key;

            // self.file.write_all(&len_byte);
            // self.file.write_all(&data_byte);
        }

        if data_block.size > 0 {
            let offset = self.file.stream_position()?;
            let block_meta = BlockMeta::new(data_block.size, offset, last_key.to_vec());
            self.index.blocks.push(block_meta);
            data_block.write_to(&mut self.file);
        }

        let index_offset = self.file.stream_position()?;

        /// writing blockMeta/index block
        // for block in self.index.blocks.iter() {
        //     let (block_meta_bytes_len_as_bytes, block_meta_bytes) = block.encode();
        //     self.file.write_all(&block_meta_bytes_len_as_bytes);
        //     self.file.write_all(&block_meta_bytes);
        // }
        self.index.write_bytes_to(&mut self.file);

        ///writing footer
        let footer_offset = self.file.stream_position()?;
        let index_len = self.file.stream_position()? - index_offset;
        let footer = Footer::new(index_offset, index_len);
        let (footer_len, footer_byte) = footer.encode();
        self.file.write_all(&footer_byte)?;
        self.file.write_all(&footer_len)?;
        // println!(
        //     "footer written: {:?}, footer len written: {}",
        //     footer,
        //     usize::from_le_bytes(footer_len)
        // );

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
        self.file.read_exact(&mut buf)?;
        let data: SkipListKV<Vec<u8>, Vec<u8>> = bitcode::decode(&buf)?;
        println!("{:?}", data);
        Ok(())
    }
}
