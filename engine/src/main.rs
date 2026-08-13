#![allow(unused)]
use std::{
    error::{self, Error},
    io::{Write, stdin, stdout},
    time::Instant,
};

use crate::{
    engine_error::EngineError,
    skiplist::{SkipList, SkipListKV, SkipListNode},
    sstable::writer::SstableWriter,
    tests::{cli, test_block_split, test_sstable_read},
    wal::Wal,
};

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
    test_block_split()?;
    test_sstable_read()?;
    // cli(skiplist)
    Ok(())
}
