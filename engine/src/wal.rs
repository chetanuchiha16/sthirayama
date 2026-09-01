use std::{
    fmt::Debug,
    fs::{self, File, OpenOptions},
    io::{Error, ErrorKind, Read, Seek, SeekFrom, Write},
    mem,
    path::{Path, PathBuf},
};

use bitcode::{DecodeOwned, Encode};

use crate::{
    memtable::Value,
    skiplist::{SkipList, SkipListKV},
    skiplist_error::{self, SkipListError},
};

#[derive(Debug)]
pub struct Wal {
    // key_len: usize,
    // key: K,
    // value_len: usize,
    // value: V,
    file: File,
    path: PathBuf, // _marker: PhantomData<(K, V)>,
}

impl Wal {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        fs::create_dir_all(&path)?;

        let wal_path = path.as_ref().join("file.wal");
        println!("{}", wal_path.display());
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&wal_path)?;

        Ok(Self {
            // key_len: size_of::<K>(),
            // value_len: size_of::<V>(),
            file,
            path: wal_path,
            // _marker: PhantomData,
        })
    }
    /// append the entry to the wal file after every insert to the skiplist
    pub fn append(&mut self, key: &Vec<u8>, value: Value) -> std::io::Result<()> {
        // let key_len_bytes = size_of::<K>().to_le_bytes();
        // let value_len_bytes = size_of::<V>().to_le_bytes();
        let data = SkipListKV::new(key.clone(), value.to_bytes());
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
        K: DecodeOwned + Clone + Debug + Encode + PartialOrd,
        V: DecodeOwned + Clone + Debug + Encode,
    >(
        &mut self,
        skiplist: &mut SkipList<Vec<u8>, Vec<u8>>,
    ) -> Result<(), skiplist_error::SkipListError> {
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

                Err(e) => return Err(e.into()),
            }

            let data_len = usize::from_le_bytes(len_buffer);

            let mut data_buffer = vec![0u8; data_len];
            self.file.read_exact(&mut data_buffer)?;
            let data: SkipListKV<Vec<u8>, Vec<u8>> = bitcode::decode(&data_buffer)?;
            // if let Value::Data(val) = Value::from_bytes(&data.value)? {
            //     println!(
            //         "{:?} : {:?}",
            //         str::from_utf8(&data.key),
            //         str::from_utf8(&val)
            //     );
            // }
            skiplist.insert(data.key, data.value);
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

    ///replace the old wal with new wal and delete the old wal
    pub fn recycle(&mut self) -> Result<(), SkipListError> {
        let old_path = &self.path;
        if let Some(old_path_parent) = old_path.parent() {
            let new_path = old_path_parent
                .join("old")
                .join(old_path.file_name().unwrap());
            println!("{}", new_path.display());
            fs::create_dir_all(&new_path.parent().unwrap())?;
            fs::rename(old_path, &new_path)?;
            let new_file = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(old_path)?;
            let old_wal = mem::replace(&mut self.file, new_file);
            drop(old_wal);
            fs::remove_file(new_path)?;
        }

        Ok(())
    }
}
