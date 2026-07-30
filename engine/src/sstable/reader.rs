use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek},
};

use crate::sstable::{
    errors::SsTableReaderError,
    footer::Footer,
    index::{BlockMeta, IndexBlock},
};

pub struct SstableReader {
    file: File,
}

impl SstableReader {
    pub fn new() -> Result<Self, SsTableReaderError> {
        let file = OpenOptions::new()
            .read(true)
            // .append(true)
            .open("table.sst")?;
        Ok(Self { file })
    }

    pub fn read_footer(&mut self) -> Result<Footer, SsTableReaderError> {
        self.file.seek(std::io::SeekFrom::End(-8));
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf);

        let len = usize::from_le_bytes(buf);
        let n: i64 = 8 + len as i64;
        self.file.seek(std::io::SeekFrom::End(-n));

        let mut buf = vec![0u8; len];
        self.file.read_exact(&mut buf);
        let ans: Footer = bitcode::decode(&buf)?;
        println!("footer len read: {len}, footer read: {:?}", ans);
        Ok(ans)
    }

    pub fn read_index(&mut self) -> Result<IndexBlock, SsTableReaderError> {
        let footer = self.read_footer()?;
        let index_offset = footer.index_offset;
        // let index_len = footer.index_len as usize; // we can just get IndexBlock len instead of from footer

        self.file.seek(std::io::SeekFrom::Start(index_offset))?;

        let mut buffer = [0u8; 8];
        self.file.read_exact(&mut buffer);
        let index_len = usize::from_le_bytes(buffer);

        let mut buf = vec![0u8; index_len];
        self.file.read_exact(&mut buf)?;
        let index: IndexBlock = bitcode::decode(&buf)?;

        // println!("here {:?}", buf);
        println!("index read: {:?}", index.blocks);
        Ok(index)
    }

    pub fn binary_search_index(
        &mut self,
        key: &Vec<u8>,
    ) -> Result<Option<usize>, SsTableReaderError> {
        let mut index_block = self.read_index()?.blocks;
        let (mut left, mut right) = (0i32, index_block.len() as i32 - 1);
        let mut ans_idx: Option<usize> = None;

        while left <= right {
            let mid = left + (right - left) / 2;
            let mid_block = &index_block[mid as usize];
            let mid_block_key = &mid_block.last_key;
            if key >= mid_block_key {
                ans_idx = Some(mid as usize);
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }

        let found = str::from_utf8(&index_block[ans_idx.unwrap() as usize].last_key)?;
        let key_val = str::from_utf8(key)?;
        let left_val = str::from_utf8(&index_block[0].last_key)?;
        let right_val = str::from_utf8(&index_block[1].last_key)?;

        println!(
            "left {}, key {}, right: {}, found: {found}",
            left_val, key_val, right_val
        );

        Ok(ans_idx)
    }
}
