#![allow(unused)]
use std::{
    error::{self, Error},
    io::{Write, stdin, stdout},
    time::Instant,
};

use crate::{
    engine::Engine,
    engine_error::EngineError,
    skiplist::{SkipList, SkipListKV, SkipListNode},
    sstable::writer::SstableWriter,
    tests::{cli, test_block_split, test_sstable_read},
    wal::Wal,
};

mod config;
mod engine;
mod engine_error;
mod memtable;
mod skiplist;
mod skiplist_error;
mod sstable;
#[cfg(test)]
mod test_skiplist;
#[cfg(test)]
mod test_sstable;
mod tests;
mod traits;
mod wal;

fn main() -> Result<(), EngineError> {
    // pring_skiplist_details()?;
    // try_new_skiplist()?;
    // try_wal()?;
    // test_block_split()?;
    // test_sstable_read()?;
    // cli(skiplist)

    let mut engine = Engine::new()?;
    for i in (0..1000) {
        let key = format!("{}", i).into_bytes();
        let value = format!("{}", i * 2).into_bytes();
        engine.set(key, value);
    }
    let key = format!("{}", 999).into_bytes();
    let val = engine.get(key)?;

    println!("{:?}", val.map(|x| { String::from_utf8(x.to_vec()) }));
    Ok(())
}
