use std::io;

#[derive(Debug)]
pub enum SkipListError {
    IoError(io::Error),
    BitcodeError(bitcode::Error),
}

impl From<io::Error> for SkipListError {
    fn from(value: io::Error) -> Self {
        SkipListError::IoError(value)
    }
}

impl From<bitcode::Error> for SkipListError {
    fn from(value: bitcode::Error) -> Self {
        SkipListError::BitcodeError(value)
    }
}
