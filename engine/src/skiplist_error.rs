use std::io;

use crate::sstable::{self, errors::SsTableWriterError};

#[derive(Debug)]
pub enum SkipListError {
    IoError(io::Error),
}

impl From<io::Error> for SkipListError {
    fn from(value: io::Error) -> Self {
        SkipListError::IoError(value)
    }
}
