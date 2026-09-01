use tempfile::NamedTempFile;

use crate::{
    engine_error::EngineError,
    memtable::{Memtable, Value},
    sstable::{reader::SstableReader, writer::SstableWriter},
};

#[test]
fn test_sstable_read_write() -> Result<(), EngineError> {
    use std::fs;

    let _ = fs::remove_file("table.sst");

    // let mut skiplist = SkipList::<Vec<u8>, Vec<u8>>::new(5, vec![b'0'], vec![b'0']);
    let mut memtable = Memtable::new();

    // Create multiple blocks
    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{}", i).into_bytes();
        memtable.insert(&key, value)?;
    }
    let file = NamedTempFile::new()?;

    let path = file.path();

    let mut writer = SstableWriter::new(path)?;
    writer.write(memtable)?;

    let mut reader = SstableReader::new(path)?;

    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let expected = format!("value{}", i).into_bytes();

        let actual = reader.binary_search_data(&key)?;

        assert_eq!(actual, Value::Data(expected));
    }

    Ok(())
}

use std::{fs, path::Path};

fn build_sstable<T: AsRef<Path>>(path: T, count: usize) -> Result<(), EngineError> {
    let _ = fs::remove_file(&path);

    // let mut skiplist = SkipList::<Vec<u8>, Vec<u8>>::new(5, vec![b'0'], vec![b'0']);
    let mut memtable = Memtable::new();

    for i in 0..count {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{}", i).into_bytes();
        memtable.insert(&key, value)?;
    }

    let mut writer = SstableWriter::new(path)?;
    writer.write(memtable)?;

    Ok(())
}

#[test]
fn test_first_key() -> Result<(), EngineError> {
    let file = NamedTempFile::new()?;

    let path = file.path();
    build_sstable(path, 1000)?;

    let mut reader = SstableReader::new(path)?;

    assert_eq!(
        reader.binary_search_data(&b"0000".to_vec())?,
        Value::Data(b"value0".to_vec())
    );

    Ok(())
}

#[test]
fn test_last_key() -> Result<(), EngineError> {
    let file = NamedTempFile::new()?;

    let path = file.path();
    build_sstable(path, 1000)?;
    let mut reader = SstableReader::new(path)?;

    assert_eq!(
        reader.binary_search_data(&b"0999".to_vec())?,
        Value::Data(b"value999".to_vec())
    );

    Ok(())
}

#[test]
fn test_middle_key() -> Result<(), EngineError> {
    let file = NamedTempFile::new()?;

    let path = file.path();
    build_sstable(path, 1000)?;
    let mut reader = SstableReader::new(path)?;

    assert_eq!(
        reader.binary_search_data(&b"0500".to_vec())?,
        Value::Data(b"value500".to_vec())
    );

    Ok(())
}

#[test]
fn test_key_not_found() -> Result<(), EngineError> {
    let file = NamedTempFile::new()?;

    let path = file.path();
    build_sstable(path, 1000)?;
    let mut reader = SstableReader::new(path)?;

    assert_eq!(reader.binary_search_data(&b"1500".to_vec())?, Value::None);

    Ok(())
}

#[test]
fn test_key_smaller_than_smallest() -> Result<(), EngineError> {
    let file = NamedTempFile::new()?;

    let path = file.path();
    build_sstable(path, 1000)?;
    let mut reader = SstableReader::new(path)?;

    assert_eq!(reader.binary_search_data(&b"-001".to_vec())?, Value::None);

    Ok(())
}

#[test]
fn test_single_entry() -> Result<(), EngineError> {
    let file = NamedTempFile::new()?;

    let path = file.path();
    build_sstable(path, 1)?;
    let mut reader = SstableReader::new(path)?;

    assert_eq!(
        reader.binary_search_data(&b"0000".to_vec())?,
        Value::Data(b"value0".to_vec())
    );

    assert_eq!(reader.binary_search_data(&b"0001".to_vec())?, Value::None);

    Ok(())
}

#[test]
fn test_update() -> Result<(), EngineError> {
    let file = NamedTempFile::new()?;
    let path = file.path();

    let mut memtable = Memtable::new();

    // Insert original value.
    let key = b"0001".to_vec();
    memtable.insert(&key, b"old_value".to_vec())?;

    // Update the same key.
    memtable.insert(&key, b"new_value".to_vec())?;

    let mut writer = SstableWriter::new(path)?;
    writer.write(memtable)?;

    let mut reader = SstableReader::new(path)?;

    assert_eq!(
        reader.binary_search_data(&key)?,
        Value::Data(b"new_value".to_vec())
    );

    Ok(())
}
