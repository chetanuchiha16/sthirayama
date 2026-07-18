use std::{
    fmt::{Debug, Display},
    fs::{File, OpenOptions},
    io::{Error, ErrorKind, Read, Seek, SeekFrom, Write},
};

use bitcode::{DecodeOwned, Encode};

use crate::{
    skiplist::{SkipList, SkipListKV},
    traits::{TypeSkipListKey, TypeSkipListValue},
};

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
    /// append the entry to the wal file after every insert to the skiplist
    pub fn append<K: TypeSkipListKey, V: TypeSkipListValue>(
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
    pub fn recover<
        K: DecodeOwned + Clone + Debug + Encode + PartialOrd + Display,
        V: DecodeOwned + Clone + Debug + Encode + Display,
    >(
        &mut self,
        skiplist: &mut SkipList<K, V>,
    ) -> Result<(), std::io::Error> {
        // let skiplist = SkipList::new(5, -1, -1).unwrap();

        self.file.seek(SeekFrom::Start(0))?;

        // let mut data_len_bytes = [0u8; 8];
        // self.file.read_exact(&mut data_len_bytes)?;
        // let data_len = usize::from_le_bytes(data_len_bytes);

        // let mut data_bytes = vec![0u8; data_len];
        // self.file.read_exact(&mut data_bytes)?;
        // let data: SkipListKV<K, V> = bitcode::decode(&data_bytes).unwrap();
        // skiplist.insert(data.key.clone(), data.value.clone())?;
        // print!("{:?}\n", data);
        // print!("{:?}\n", skiplist);

        loop {
            let mut len_buffer = [0u8; 8];
            match self.file.read_exact(&mut len_buffer) {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,

                Err(e) => return Err(e),
            }
            let data_len = usize::from_le_bytes(len_buffer);

            let mut data_buffer = vec![0u8; data_len];
            self.file.read_exact(&mut data_buffer)?;
            let data: SkipListKV<K, V> = bitcode::decode(&data_buffer).unwrap();
            println!("{} : {}", data.key, data.value);
            skiplist.insert(data.key, data.value)?;
        }
        // println!("{}", skiplist);
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
