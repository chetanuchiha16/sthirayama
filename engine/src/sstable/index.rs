#[derive(Debug)]
pub struct BlockMeta {
    len: usize,
    offset: usize,
    last_key: Vec<u8>,
}

impl BlockMeta {
    pub fn new(len: usize, offset: usize, last_key: Vec<u8>) -> Self {
        Self {
            len,
            offset,
            last_key,
        }
    }
}
