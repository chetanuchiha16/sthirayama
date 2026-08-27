use bitcode::{Decode, Encode};

use crate::{skiplist::SkipList, skiplist_error::SkipListError};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum Value {
    Data(Vec<u8>),
    Tombstone,
    None,
}

impl Value {
    pub fn to_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bitcode::Error> {
        bitcode::decode(bytes)
    }
}

pub struct Memtable {
    pub skiplist: SkipList<Vec<u8>, Vec<u8>>,
    pub size: usize,
}

impl Memtable {
    pub fn new() -> Self {
        Self {
            skiplist: SkipList::new(5, b"0".to_vec(), b"0".to_vec()),
            size: 0,
        }
    }

    pub fn insert(&mut self, key: &Vec<u8>, value: Vec<u8>) -> Result<(), SkipListError> {
        let val = Value::Data(value.to_vec()).to_bytes();
        // let val_bytes = val.to_bytes();
        self.size += key.len() + value.len();
        // println!("{}", self.size);
        self.skiplist.insert(key.clone(), val);
        Ok(())
    }

    pub fn extract(&self, key: &Vec<u8>) -> Result<Value, SkipListError> {
        // println!("extract memtable");
        let bytes = self.skiplist.search(key.to_vec());
        match bytes {
            Some(bytes) => {
                // println!("found in memtable");
                let k = Value::from_bytes(&bytes)?;
                // println!("{:?}", k);
                // match k {
                //     Value::Data(val) =>  Ok(Some(val)),
                //     Tombstone =>  Ok(None),
                // }
                // Ok(Some(bytes))
                Ok(k)
            }
            None => Ok(Value::None),
        }
        // Ok(bytes)
    }
    ///deletes a key, ie marks the key as tombstone 
    pub fn delete(&mut self, key: &Vec<u8>) {
        self.skiplist
            .insert(key.clone(), Value::Tombstone.to_bytes());
    }
}
