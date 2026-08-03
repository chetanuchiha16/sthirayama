use crate::{
    engine_error::EngineError,
    skiplist::SkipList,
    sstable::{reader::SstableReader, writer::SstableWriter},
};

#[test]
fn test_sstable_read_write() -> Result<(), EngineError> {
    use std::fs;

    let _ = fs::remove_file("table.sst");

    let mut skiplist = SkipList::<Vec<u8>, Vec<u8>>::new(5, vec![b'0'], vec![b'0'])?;

    // Create multiple blocks
    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{}", i).into_bytes();
        skiplist.insert_with_wal(key, value);
    }

    let mut writer = SstableWriter::new(skiplist)?;
    writer.write()?;

    let mut reader = SstableReader::new()?;

    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let expected = format!("value{}", i).into_bytes();

        let actual = reader.binary_search_data(&key)?;

        assert_eq!(actual, Some(expected));
    }

    Ok(())
}

use std::fs;

fn build_sstable(count: usize) -> Result<(), EngineError> {
    let _ = fs::remove_file("table.sst");

    let mut skiplist = SkipList::<Vec<u8>, Vec<u8>>::new(5, vec![b'0'], vec![b'0'])?;

    for i in 0..count {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{}", i).into_bytes();
        skiplist.insert_with_wal(key, value);
    }

    let mut writer = SstableWriter::new(skiplist)?;
    writer.write()?;

    Ok(())
}

#[test]
fn test_first_key() -> Result<(), EngineError> {
    build_sstable(1000)?;

    let mut reader = SstableReader::new()?;

    assert_eq!(
        reader.binary_search_data(&b"0000".to_vec())?,
        Some(b"value0".to_vec())
    );

    Ok(())
}

#[test]
fn test_last_key() -> Result<(), EngineError> {
    build_sstable(1000)?;

    let mut reader = SstableReader::new()?;

    assert_eq!(
        reader.binary_search_data(&b"0999".to_vec())?,
        Some(b"value999".to_vec())
    );

    Ok(())
}

#[test]
fn test_middle_key() -> Result<(), EngineError> {
    build_sstable(1000)?;

    let mut reader = SstableReader::new()?;

    assert_eq!(
        reader.binary_search_data(&b"0500".to_vec())?,
        Some(b"value500".to_vec())
    );

    Ok(())
}

#[test]
fn test_key_not_found() -> Result<(), EngineError> {
    build_sstable(1000)?;

    let mut reader = SstableReader::new()?;

    assert_eq!(reader.binary_search_data(&b"1500".to_vec())?, None);

    Ok(())
}

#[test]
fn test_key_smaller_than_smallest() -> Result<(), EngineError> {
    build_sstable(1000)?;

    let mut reader = SstableReader::new()?;

    assert_eq!(reader.binary_search_data(&b"-001".to_vec())?, None);

    Ok(())
}

#[test]
fn test_single_entry() -> Result<(), EngineError> {
    build_sstable(1)?;

    let mut reader = SstableReader::new()?;

    assert_eq!(
        reader.binary_search_data(&b"0000".to_vec())?,
        Some(b"value0".to_vec())
    );

    assert_eq!(reader.binary_search_data(&b"0001".to_vec())?, None);

    Ok(())
}

#[test]
fn test_empty_table() -> Result<(), EngineError> {
    build_sstable(0)?;

    let mut reader = SstableReader::new()?;

    assert_eq!(reader.binary_search_data(&b"0000".to_vec())?, None);

    Ok(())
}
