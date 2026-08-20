#![allow(unused)]
use std::{
    error::{self, Error},
    io::{Write, stdin, stdout},
    time::Instant,
};

use engine::{engine::Engine, engine_error::EngineError};


fn main() -> Result<(), EngineError> {
    // pring_skiplist_details()?;
    // try_new_skiplist()?;
    // try_wal()?;
    // test_block_split()?;
    // test_sstable_read()?;
    // cli(skiplist)

    let mut engine = Engine::new("main")?;
    for i in (0..10000) {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("{:04}", i * 2).into_bytes();
        engine.set(&key, value)?;
    }
    for i in (0..10000) {
        let key = format!("{:04}", i).into_bytes(); // to make sure lex sort == num sort
        let val = engine.get(&key)?;

        println!(
            "finding {}, {:?} from main",
            String::from_utf8(key.to_vec())?,
            val.map(|x| { String::from_utf8(x.to_vec()) })
        );
    }

    Ok(())
}
