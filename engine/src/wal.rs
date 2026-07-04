use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{Error, Write},
};

#[derive(Debug)]
pub struct Wal {
    file: File,
}

impl Wal {
    pub fn new() -> Result<Self, Error> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("../wal/file.wal")?;

        Ok(Self { file })
    }
    pub fn append<K: Display, V: Display>(&mut self, key: K, value: V) -> std::io::Result<()> {
        writeln!(self.file, "{} {}", key, value)?;
        Ok(())
    }
}
