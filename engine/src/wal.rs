use std::{
    fmt::Display, fs::{File, OpenOptions}, io::{Error, Write}, marker::PhantomData,
};

#[derive(Debug)]
pub struct Wal<K, V> {
    key_len: usize,
    // key: K,
    value_len: usize,
    // value: V,
    file: File,
    _marker: PhantomData<(K, V)>
}

impl<K: Display, V: Display> Wal<K, V> {
    pub fn new() -> Result<Self, Error> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("../wal/file.wal")?;

        Ok(Self {
            key_len: size_of::<K>(),
            value_len: size_of::<V>(),
            file,
            _marker: PhantomData
        })
    }
    pub fn append(&mut self, key: K, value: V) -> std::io::Result<()> {
        writeln!(
            self.file,
            "[{}][{}][{}][{}]",
            self.key_len, key, self.value_len, value
        )?;
        Ok(())
    }
}
