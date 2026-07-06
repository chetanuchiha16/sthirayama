use std::{
    fmt::Display, fs::{File, OpenOptions}, io::{Error, Read, Seek, SeekFrom, Write}, marker::PhantomData,
};

use bitcode::Encode;

#[derive(Debug)]
pub struct Wal<K, V> {
    // key_len: usize,
    // key: K,
    // value_len: usize,
    // value: V,
    file: File,
    _marker: PhantomData<(K, V)>,
}

impl<K: Display + Encode, V: Display + Encode> Wal<K, V> {
    pub fn new() -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("../wal/file.wal")?;

        Ok(Self {
            // key_len: size_of::<K>(),
            // value_len: size_of::<V>(),
            file,
            _marker: PhantomData,
        })
    }

    pub fn append(&mut self, key: K, value: V) -> std::io::Result<()> {
        // let key_len_bytes = size_of::<K>().to_le_bytes();
        // let value_len_bytes = size_of::<V>().to_le_bytes();
        let key_bytes = bitcode::encode(&key);
        let value_bytes = bitcode::encode(&value);
        let key_len_bytes = key_bytes.len().to_le_bytes();
        let value_len_bytes = value_bytes.len().to_le_bytes();
        // writeln!(
        //     self.file,
        //     "{:?}{:?}{:?}{:?}",
        //     key_len_bytes, key_bytes, value_len_bytes, value_bytes
        // )?;

        self.file.write_all(&key_len_bytes)?;
        self.file.write_all(&key_bytes)?;
        self.file.write_all(&value_len_bytes)?;
        self.file.write_all(&value_bytes)?;

        self.file.flush()?;
        Ok(())
    }

    pub fn recover(&mut self) -> Result<(), std::io::Error> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;
        println!("{:?} is the buf", buf);
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;
        println!("{:?} is the buf", buf);
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;
        println!("{:?} is the buf", buf);
        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;
        println!("{:?} is the buf", buf);
        Ok(())
    }
}
