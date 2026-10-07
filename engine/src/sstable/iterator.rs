use std::fs::File;

use crate::{
    skiplist::SkipListKV,
    sstable::{data_block::DataBlock, index::IndexBlock},
};

pub struct SstableIterator<'a> {
    pub file: &'a mut File,
    pub index: IndexBlock,
    pub current_entry: usize,
    pub current_block: usize,
    // pub data_block: Vec<SkipListKV<Vec<u8>, Vec<u8>>>,
}

// impl <'a>SstableIterator<'a> {
//     pub fn new(
//         file: &mut File,
//         index: IndexBlock,
//         current_entry: usize,
//         current_block: usize,
//     ) -> Self {
//         let data_block: Vec<SkipListKV<Vec<u8>, Vec<u8>>> =
//             DataBlock::read(&mut file, &index.blocks[0]).unwrap();
//         Self {
//             file: &'a mut file,
//             index,
//             current_entry,
//             current_block,
//             data_block,
//         }
//     }

// fn read_block(&self, index: u64, block_meta: BlockMeta) -> SkipListKV<Vec<u8>, Vec<u8>> {
//     let block_offset = block_meta.offset;
//     let block_len = block_meta.len;

//     SkipListKV::new(k_bytes.clone(), v_bytes.clone())
// }
// }

impl<'a> Iterator for SstableIterator<'a> {
    type Item = SkipListKV<Vec<u8>, Vec<u8>>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // println!("block {}", self.current_block);
            if self.current_block >= self.index.blocks.len()
            // || self.current_entry >= data_block.len()
            {
                return None;
            }
            let block_meta = &self.index.blocks[self.current_block];
            let data_block = DataBlock::read(&mut self.file, block_meta).unwrap();
            println!("data block size : {}", data_block.len());
            // if self.current_entry > block_len {
            if self.current_entry >= data_block.len() {
                println!("Block {}", self.current_block);
                self.current_block += 1;
                // self.data_block =
                self.current_entry = 0;
                continue;
            }
            self.current_entry += 1;
            return Some(data_block[self.current_entry - 1].clone());
        }
    }
}
