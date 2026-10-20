use std::{io, net::AddrParseError, num::ParseIntError, str::Utf8Error, string::FromUtf8Error};

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
    ParseIntError(ParseIntError),
    AddrParseError(AddrParseError),
    TransportError(tonic::transport::Error),
}
impl From<tonic::transport::Error> for EngineError {
    fn from(value: tonic::transport::Error) -> Self {
        EngineError::TransportError(value)
    }
}
impl From<AddrParseError> for EngineError {
    fn from(value: AddrParseError) -> Self {
        EngineError::AddrParseError(value)
    }
}
impl From<ParseIntError> for EngineError {
    fn from(value: ParseIntError) -> Self {
        EngineError::ParseIntError(value)
    }
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
