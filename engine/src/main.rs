#![allow(unused)]
use std::{
    error::{self, Error},
    io::{Write, stdin, stdout},
    time::Instant,
};

use crate::{
    skiplist::{SkipList, SkipListKV, SkipListNode},
    sstable::writer::SstableWriter,
    tests::{cli, test_block_split},
    wal::Wal,
};
mod skiplist;
mod sstable;
#[cfg(test)]
mod test_skiplist;
mod tests;
mod wal;

fn main() -> Result<(), Box<dyn Error>> {
    // pring_skiplist_details()?;
    // try_new_skiplist()?;
    // try_wal()?;
    test_block_split();
    // cli(skiplist)
    Ok(())
}
