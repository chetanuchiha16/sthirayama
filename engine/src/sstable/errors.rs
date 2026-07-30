use std::io;

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
    // Error(Error)
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
