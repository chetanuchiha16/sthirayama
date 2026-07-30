use std::{io, str::Utf8Error};

#[derive(Debug)]
pub enum SsTableWriterError {
    IoError(io::Error),
    BitcodeError(bitcode::Error),
    // Error(Error)
}

impl From<io::Error> for SsTableWriterError {
    fn from(value: io::Error) -> Self {
        SsTableWriterError::IoError(value)
    }
}
impl From<bitcode::Error> for SsTableWriterError {
    fn from(value: bitcode::Error) -> Self {
        SsTableWriterError::BitcodeError(value)
    }
}

#[derive(Debug)]
pub enum SsTableReaderError {
    IoError(io::Error),
    BitcodeError(bitcode::Error),
    Utf8Error(Utf8Error), // Error(Error)
}

impl From<io::Error> for SsTableReaderError {
    fn from(value: io::Error) -> Self {
        SsTableReaderError::IoError(value)
    }
}
impl From<bitcode::Error> for SsTableReaderError {
    fn from(value: bitcode::Error) -> Self {
        SsTableReaderError::BitcodeError(value)
    }
}

impl From<Utf8Error> for SsTableReaderError {
    fn from(value: Utf8Error) -> Self {
        SsTableReaderError::Utf8Error(value)
    }
}
