use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek},
};

use crate::sstable::errors::SsTableReaderError;

pub struct SstableReader {
    file: File,
}

impl SstableReader {
    pub fn new() -> Result<Self, SsTableReaderError> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open("table.sst")?;
        Ok(Self { file })
    }

    pub fn read_footer(&mut self) -> Result<(), SsTableReaderError> {
        self.file.seek(std::io::SeekFrom::End(-8));
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf);

        let len = usize::from_le_bytes(buf);
        let n: i64 = 8 + len as i64;
        self.file.seek(std::io::SeekFrom::End(-n));

        let mut buf = vec![0u8; len];
        self.file.read_exact(&mut buf);
        let ans: u64 = bitcode::decode(&buf)?;
        println!("offset len read from footer {}", len);
        println!("index offset read from footer {}", ans);
        Ok(())
    }
}
