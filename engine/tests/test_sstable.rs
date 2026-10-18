use std::path::{Path, PathBuf};
use tempfile::tempdir;

use engine::{
    engine_error::EngineError,
    memtable::{Memtable, Value},
    sstable::{reader::SstableReader, writer::SstableWriter},
};

#[test]
fn test_sstable_read_write() -> Result<(), EngineError> {
    let mut memtable = Memtable::new();

    // Create multiple blocks
    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{}", i).into_bytes();
        memtable.insert(&key, value, 0)?;
    }
    let dir = tempdir()?;
    let path = dir.path();
    let file_no = 0;

    let mut writer = SstableWriter::new(path, file_no)?;
    writer.write(memtable.skiplist.iter())?;

    let sst_path = path.join(format!("{:06}.sst", file_no));
    let mut reader = SstableReader::new(sst_path, 0)?;

    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let expected = format!("value{}", i).into_bytes();

        let actual = reader.binary_search_data(&key)?;

        assert_eq!(actual, Value::Data(expected));
    }

    Ok(())
}

fn build_sstable<T: AsRef<Path>>(dir: T, count: usize) -> Result<PathBuf, EngineError> {
    let mut memtable = Memtable::new();

    for i in 0..count {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{}", i).into_bytes();
        memtable.insert(&key, value, 0)?;
    }

    let file_no = 0;
    let mut writer = SstableWriter::new(&dir, file_no)?;
    writer.write(memtable.skiplist.iter())?;

    Ok(dir.as_ref().join(format!("{:06}.sst", file_no)))
}

#[test]
fn test_first_key() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let sst_path = build_sstable(dir.path(), 1000)?;

    let mut reader = SstableReader::new(sst_path, 0)?;

    assert_eq!(
        reader.binary_search_data(&b"0000".to_vec())?,
        Value::Data(b"value0".to_vec())
    );

    Ok(())
}

#[test]
fn test_last_key() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let sst_path = build_sstable(dir.path(), 1000)?;
    let mut reader = SstableReader::new(sst_path, 0)?;

    assert_eq!(
        reader.binary_search_data(&b"0999".to_vec())?,
        Value::Data(b"value999".to_vec())
    );

    Ok(())
}

#[test]
fn test_middle_key() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let sst_path = build_sstable(dir.path(), 1000)?;
    let mut reader = SstableReader::new(sst_path, 0)?;

    assert_eq!(
        reader.binary_search_data(&b"0500".to_vec())?,
        Value::Data(b"value500".to_vec())
    );

    Ok(())
}

#[test]
fn test_key_not_found() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let sst_path = build_sstable(dir.path(), 1000)?;
    let mut reader = SstableReader::new(sst_path, 0)?;

    assert_eq!(reader.binary_search_data(&b"1500".to_vec())?, Value::None);

    Ok(())
}

#[test]
fn test_key_smaller_than_smallest() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let sst_path = build_sstable(dir.path(), 1000)?;
    let mut reader = SstableReader::new(sst_path, 0)?;

    assert_eq!(reader.binary_search_data(&b"-001".to_vec())?, Value::None);

    Ok(())
}

#[test]
fn test_single_entry() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let sst_path = build_sstable(dir.path(), 1)?;
    let mut reader = SstableReader::new(sst_path, 0)?;

    assert_eq!(
        reader.binary_search_data(&b"0000".to_vec())?,
        Value::Data(b"value0".to_vec())
    );

    assert_eq!(reader.binary_search_data(&b"0001".to_vec())?, Value::None);

    Ok(())
}

#[test]
fn test_update() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let path = dir.path();
    let file_no = 0;

    let mut memtable = Memtable::new();

    // Insert original value.
    let key = b"0001".to_vec();
    memtable.insert(&key, b"old_value".to_vec(), 0)?;

    // Update the same key.
    memtable.insert(&key, b"new_value".to_vec(), 0)?;

    let mut writer = SstableWriter::new(path, file_no)?;
    writer.write(memtable.skiplist.iter())?;

    let sst_path = path.join(format!("{:06}.sst", file_no));
    let mut reader = SstableReader::new(sst_path, 0)?;

    assert_eq!(
        reader.binary_search_data(&key)?,
        Value::Data(b"new_value".to_vec())
    );

    Ok(())
}
