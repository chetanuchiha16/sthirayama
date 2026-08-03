use std::sync::atomic::AtomicUsize;

pub mod data_block;
pub mod errors;
pub mod footer;
pub mod index;
pub mod manifest;
pub mod reader;
pub mod writer;

static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);
