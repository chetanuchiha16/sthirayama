use std::fs::{File, OpenOptions};

pub struct SstableWriter {
    file: File,
}

impl SstableWriter {
    pub fn new() -> Self {
        let file = OpenOptions::new().create(true).open("table.sst").unwrap();
        Self { file }
    }
    pub fn write(&self) {}
}
