use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
    sync::atomic::Ordering,
};

use crate::sstable::GLOBAL_COUNT;

pub struct Manifest {
    pub file: File,
}

impl Manifest {
    pub fn new() -> Result<Self, io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open("MANIFEST")?;
        Ok(Self { file: file })
    }

    pub fn read(&mut self) -> Result<String, io::Error> {
        self.file.seek(std::io::SeekFrom::End(-8))?;
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;
        println!(
            "{}, {}",
            usize::from_le_bytes(buf),
            GLOBAL_COUNT.load(Ordering::SeqCst)
        );

        let _buf = GLOBAL_COUNT.load(Ordering::SeqCst).to_le_bytes();

        let path = format!("sstable/{:06}.sst", GLOBAL_COUNT.load(Ordering::SeqCst));
        GLOBAL_COUNT.fetch_sub(1, Ordering::SeqCst);
        Ok(path)
    }

    pub fn write(&mut self) -> Result<String, io::Error> {
        let path = format!("sstable/{:06}.sst", GLOBAL_COUNT.load(Ordering::SeqCst));

        let buf = GLOBAL_COUNT.load(Ordering::SeqCst).to_le_bytes();
        println!(
            "{}, {}",
            usize::from_le_bytes(buf),
            GLOBAL_COUNT.load(Ordering::SeqCst)
        );

        self.file.write_all(&buf)?;
        self.file.flush()?;
        GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst);
        Ok(path)
    }
}
