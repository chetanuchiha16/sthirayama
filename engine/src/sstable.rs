use std::{
    fs::{File, OpenOptions},
    io,
};

use crate::skiplist::{self, SkipList};

pub struct SstableWriter {
    file: File,
    skiplist: SkipList<Vec<u8>, Vec<u8>>,
}

impl SstableWriter {
    pub fn new(skiplist: SkipList<Vec<u8>, Vec<u8>>) -> Result<Self, io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("table.sst")?;
        Ok(Self { file, skiplist })
    }
    pub fn write(&self) {}
}
