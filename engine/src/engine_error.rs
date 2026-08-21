use std::{io, str::Utf8Error, string::FromUtf8Error};

use crate::{
    skiplist_error::{self, SkipListError},
    sstable::errors::{SsTableReaderError, SsTableWriterError},
};

#[derive(Debug)]
pub enum EngineError {
    SkipListError(skiplist_error::SkipListError),
    SsTableWriterError(SsTableWriterError),
    SsTableReaderError(SsTableReaderError),
    FromutfError(FromUtf8Error),
    IoError(io::Error),
    BitcodeError(bitcode::Error),
}

impl From<bitcode::Error> for EngineError {
    fn from(value: bitcode::Error) -> Self {
        EngineError::BitcodeError(value)
    }
}
impl From<SkipListError> for EngineError {
    fn from(value: SkipListError) -> Self {
        EngineError::SkipListError(value)
    }
}

impl From<SsTableWriterError> for EngineError {
    fn from(value: SsTableWriterError) -> Self {
        EngineError::SsTableWriterError(value)
    }
}

impl From<SsTableReaderError> for EngineError {
    fn from(value: SsTableReaderError) -> Self {
        EngineError::SsTableReaderError(value)
    }
}

impl From<FromUtf8Error> for EngineError {
    fn from(value: FromUtf8Error) -> Self {
        EngineError::FromutfError(value)
    }
}

impl From<io::Error> for EngineError {
    fn from(value: io::Error) -> Self {
        EngineError::IoError(value)
    }
}

impl From<Utf8Error> for EngineError {
    fn from(value: Utf8Error) -> Self {
        EngineError::SsTableReaderError(SsTableReaderError::Utf8Error(value))
    }
}
