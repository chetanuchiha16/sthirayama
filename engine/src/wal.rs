use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{Error, Read, Seek, SeekFrom, Write},
};

use bitcode::{DecodeOwned, Encode};

use crate::skiplist::SkipListKV;

#[derive(Debug)]
pub struct Wal {
    // key_len: usize,
    // key: K,
    // value_len: usize,
    // value: V,
    file: File,
    // _marker: PhantomData<(K, V)>,
}

impl Wal {
    pub fn new() -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("wal/file.wal")?;

        Ok(Self {
            // key_len: size_of::<K>(),
            // value_len: size_of::<V>(),
            file,
            // _marker: PhantomData,
        })
    }

    pub fn append<K: Encode + Clone, V: Encode + Clone>(
        &mut self,
        key: K,
        value: V,
    ) -> std::io::Result<()> {
        // let key_len_bytes = size_of::<K>().to_le_bytes();
        // let value_len_bytes = size_of::<V>().to_le_bytes();
        let data = SkipListKV::new(key, value);
        let data_bytes = bitcode::encode(&data);
        let data_len_bytes = data_bytes.len().to_le_bytes();
        self.file.write_all(&data_len_bytes)?;
        self.file.write_all(&data_bytes)?;
        // let key_bytes = bitcode::encode(&data.key);
        // let value_bytes = bitcode::encode(&data.value);
        // let key_len_bytes = key_bytes.len().to_le_bytes();
        // let value_len_bytes = value_bytes.len().to_le_bytes();
        // // writeln!(
        // //     self.file,
        // //     "{:?}{:?}{:?}{:?}",
        // //     key_len_bytes, key_bytes, value_len_bytes, value_bytes
        // // )?;

        // self.file.write_all(&key_len_bytes)?;
        // self.file.write_all(&key_bytes)?;
        // self.file.write_all(&value_len_bytes)?;
        // self.file.write_all(&value_bytes)?;

        self.file.flush()?;
        Ok(())
    }

    ///recover the skip list from the wal file if crashed
    pub fn recover<K: DecodeOwned + Clone + Debug, V: DecodeOwned + Clone + Debug>(
        &mut self,
    ) -> Result<(), std::io::Error> {
        self.file.seek(SeekFrom::Start(0))?;

        let mut buf = [0u8; 8];
        self.file.read_exact(&mut buf)?;
        let data_len = usize::from_le_bytes(buf);

        let mut buf = vec![0u8; data_len];
        self.file.read_exact(&mut buf)?;
        let data: SkipListKV<K, V> = bitcode::decode(&buf).unwrap();

        print!("{:?}\n", data);
        // let mut buf = [0u8; 8];
        // self.file.read_exact(&mut buf)?;
        // let key_len = usize::from_le_bytes(buf);
        // // println!("{}", &key_len);
        // let mut buf = vec![0u8; key_len];
        // self.file.read_exact(&mut buf)?;
        // let key: K = bitcode::decode(&buf).unwrap();
        // // println!("{:?}", key);

        // let mut buf = [0u8; 8];
        // self.file.read_exact(&mut buf)?;
        // let value_len = usize::from_le_bytes(buf);
        // // println!("{}", &value_len);
        // let mut buf = vec![0u8; value_len];
        // self.file.read_exact(&mut buf)?;
        // let value: V = bitcode::decode(&buf).unwrap();
        // println!("{:?}: {:?}",key, value);
        // // let item: usize = bitcode::decode(&buf).unwrap();
        // // println!("{:?}", item);

        // // println!("{:?} is the buf", buf);
        // // let mut buf = [0u8; 8];
        // // self.file.read_exact(&mut buf)?;
        // // println!("{:?} is the buf", buf);
        // // let mut buf = [0u8; 8];
        // // self.file.read_exact(&mut buf)?;
        // // println!("{:?} is the buf", buf);
        // // let mut buf = [0u8; 8];
        // // self.file.read_exact(&mut buf)?;
        // // println!("{:?} is the buf", buf);
        Ok(())
    }
}
