use std::{io, string::FromUtf8Error};

use crate::{
    skiplist_error::{self, SkipListError},
    sstable::errors::SsTableWriterError,
};

pub enum EngineError {
    SkipListError(skiplist_error::SkipListError),
    SsTableWriterError(SsTableWriterError),
    FromutfError(FromUtf8Error),
    IoError(io::Error),
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
