use engine::{engine::Engine, sstable::reader::SstableReader};
use tempfile::tempdir;

#[test]
fn test_set_and_get_from_memtable() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"0001".to_vec();
    let value = b"value1".to_vec();

    engine.set(&key, &value).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(value));
}

#[test]
fn test_get_missing_key() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"9999".to_vec();

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[tokio::test]
async fn test_flush_and_get_from_sstable() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    // Make the memtable exceed 4 KB.
    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, &value).unwrap();
    }

    // These should now be in an SSTable.
    let key = b"0000".to_vec();
    assert_eq!(engine.get(&key).unwrap(), Some(b"value0000".to_vec()));

    let key = b"0500".to_vec();
    assert_eq!(engine.get(&key).unwrap(), Some(b"value0500".to_vec()));

    let key = b"0999".to_vec();
    assert_eq!(engine.get(&key).unwrap(), Some(b"value0999".to_vec()));
}

#[tokio::test]
async fn test_flush_multiple_sstables() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    // Enough data to create multiple SSTables.
    for i in 0..5000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, &value).unwrap();
    }

    // Beginning of the first SSTable.
    assert_eq!(
        engine.get(&b"0000".to_vec()).unwrap(),
        Some(b"value0000".to_vec())
    );

    // Somewhere in the middle.
    assert_eq!(
        engine.get(&b"2500".to_vec()).unwrap(),
        Some(b"value2500".to_vec())
    );

    // End of the data.
    assert_eq!(
        engine.get(&b"4999".to_vec()).unwrap(),
        Some(b"value4999".to_vec())
    );
}

#[tokio::test]
async fn test_key_not_found_after_flush() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, &value).unwrap();
    }

    assert_eq!(engine.get(&b"9999".to_vec()).unwrap(), None);
}

#[tokio::test]
async fn test_keys_across_multiple_flushes() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    for i in 0..10000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("{:04}", i * 2).into_bytes();

        engine.set(&key, &value).unwrap();
    }

    for i in 0..10000 {
        let key = format!("{:04}", i).into_bytes();
        let expected = format!("{:04}", i * 2).into_bytes();

        assert_eq!(
            engine.get(&key).unwrap(),
            Some(expected),
            "failed for key {:04}",
            i
        );
    }
}

#[test]
fn test_set_overwrites_value() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, &b"first".to_vec()).unwrap();
    assert_eq!(engine.get(&key).unwrap(), Some(b"first".to_vec()));

    // Update the existing key.
    engine.set(&key, &b"second".to_vec()).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(b"second".to_vec()));
}

#[tokio::test]
async fn test_update_after_flush() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, &b"first".to_vec()).unwrap();

    // Force the original value into an SSTable.
    for i in 0..1000 {
        let key = format!("{:04}", i + 10).into_bytes();
        let value = format!("value{:04}", i + 10).into_bytes();
        engine.set(&key, &value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"first".to_vec()));

    // New value goes into the newer memtable.
    engine.set(&key, &b"second".to_vec()).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(b"second".to_vec()));
}

#[tokio::test]
async fn test_delete_from_memtable() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, &b"value1".to_vec()).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(b"value1".to_vec()));

    engine.del(&key).unwrap();

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[tokio::test]
async fn test_delete_after_flush() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, &b"value1".to_vec()).unwrap();

    // Force the value into an SSTable.
    for i in 0..1000 {
        let key = format!("{:04}", i + 10).into_bytes();
        let value = format!("value{:04}", i + 10).into_bytes();

        engine.set(&key, &value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"value1".to_vec()));

    // Tombstone goes into the newer MemTable.
    engine.del(&key).unwrap();

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[tokio::test]
async fn test_delete_survives_flush() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, &b"value1".to_vec()).unwrap();

    // Flush original value.
    for i in 0..1000 {
        let key = format!("{:04}", i + 10).into_bytes();
        let value = format!("value{:04}", i + 10).into_bytes();

        engine.set(&key, &value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"value1".to_vec()));

    // Write tombstone.
    engine.del(&key).unwrap();

    // Force tombstone into an SSTable too.
    for i in 0..1000 {
        let key = format!("{:04}", i + 2000).into_bytes();
        let value = format!("value{:04}", i + 2000).into_bytes();

        engine.set(&key, &value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[test]
fn test_delete_missing_key() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"9999".to_vec();

    assert_eq!(engine.get(&key).unwrap(), None);

    engine.del(&key).unwrap();

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[tokio::test]
async fn test_overwrite_across_multiple_flushes() {
    let dir = tempdir().unwrap();
    let mut engine = Engine::new(dir.path()).unwrap();

    let key = b"0001".to_vec();

    // Set key to "first_value" and flush to SSTable 0.
    engine.set(&key, &b"first_value".to_vec()).unwrap();
    for i in 0..1000 {
        let k = format!("{:04}", i + 10).into_bytes();
        let v = format!("value{:04}", i + 10).into_bytes();
        engine.set(&k, &v).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"first_value".to_vec()));

    // Overwrite key with "second_value" and flush to SSTable 1.
    engine.set(&key, &b"second_value".to_vec()).unwrap();
    for i in 0..1000 {
        let k = format!("{:04}", i + 2000).into_bytes();
        let v = format!("value{:04}", i + 2000).into_bytes();
        engine.set(&k, &v).unwrap();
    }

    // Newer SSTable value should override older SSTable value.
    assert_eq!(engine.get(&key).unwrap(), Some(b"second_value".to_vec()));
}

#[tokio::test]
async fn test_engine_reopen_persistence() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    let key = b"0001".to_vec();
    let value = b"persistent_value".to_vec();

    {
        let mut engine = Engine::new(db_path).unwrap();
        engine.set(&key, &value).unwrap();

        // Flush to SSTable.
        for i in 0..1000 {
            let k = format!("{:04}", i + 10).into_bytes();
            let v = format!("value{:04}", i + 10).into_bytes();
            engine.set(&k, &v).unwrap();
        }
        assert_eq!(engine.get(&key).unwrap(), Some(value.clone()));
    }

    // Re-open Engine at the same directory.
    let mut reopened_engine = Engine::new(db_path).unwrap();
    assert_eq!(reopened_engine.get(&key).unwrap(), Some(value));
}

#[tokio::test]
async fn test_sequence_recovery_unflushed_wal() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    {
        let mut engine = Engine::new(db_path).unwrap();
        // Insert key with alphabetically high name first (sequence = 1)
        engine.set(&b"z_key".to_vec(), &b"val1".to_vec()).unwrap();
        // Insert key with alphabetically lower name second (sequence = 2)
        engine.set(&b"a_key".to_vec(), &b"val2".to_vec()).unwrap();
        // Drop engine without exceeding 4KB (data stays in WAL, not flushed to SSTable)
    }

    // Reopen engine from the same path
    let mut reopened = Engine::new(db_path).unwrap();

    // Now insert a new key after recovery and flush to SSTable
    reopened
        .set(&b"b_new_key".to_vec(), &b"val3".to_vec())
        .unwrap();
    for i in 0..1000 {
        let k = format!("{:04}", i + 10).into_bytes();
        let v = format!("value{:04}", i + 10).into_bytes();
        reopened.set(&k, &v).unwrap();
    }

    // Read the flushed SSTable (file 0)
    let sstable = SstableReader::new(db_path, 0).unwrap();
    let entries: Vec<_> = sstable.iter().unwrap().collect();

    let new_entry = entries
        .iter()
        .find(|kv| kv.key == b"b_new_key".as_slice())
        .expect("b_new_key should be in SSTable");

    let new_seq = usize::from_le_bytes(new_entry.sequence);
    // Since "z_key" had seq 1, and "a_key" had seq 2, the next insert "b_new_key"
    // must have sequence 3 (or higher).
    assert!(
        new_seq >= 3,
        "Expected new write sequence to be >= 3, but got {}",
        new_seq
    );
}

#[tokio::test]
async fn test_sequence_recovery_after_sstable_flush() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    {
        let mut engine = Engine::new(db_path).unwrap();
        // Insert 350 items (~4.5KB) to trigger a single flush into SSTable 0 (sequences 1..=350)
        for i in 0..350 {
            let k = format!("{:04}", i).into_bytes();
            let v = format!("value{:04}", i).into_bytes();
            engine.set(&k, &v).unwrap();
        }
        // At this point, SSTable 0 has sequences up to 350, and WAL was recycled.
    }

    // Reopen engine
    let mut reopened = Engine::new(db_path).unwrap();

    // Insert key after reopen and another 350 items to trigger flush into SSTable 1
    reopened
        .set(&b"after_reopen_key".to_vec(), &b"val".to_vec())
        .unwrap();
    for i in 0..350 {
        let k = format!("new_{:04}", i).into_bytes();
        let v = format!("new_value{:04}", i).into_bytes();
        reopened.set(&k, &v).unwrap();
    }

    // Read SSTable 1
    let sstable = SstableReader::new(db_path, 1).unwrap();
    let entries: Vec<_> = sstable.iter().unwrap().collect();

    let entry = entries
        .iter()
        .find(|kv| kv.key == b"after_reopen_key".as_slice())
        .expect("after_reopen_key should be in SSTable 1");

    let new_seq = usize::from_le_bytes(entry.sequence);
    // The sequence must be greater than 350 (the sequence of pre-reopen items)
    assert!(
        new_seq > 350,
        "Expected sequence to be > 350, but got {}",
        new_seq
    );
}

#[tokio::test]
async fn test_sequence_recovery_mixed_sstable_and_wal() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    {
        let mut engine = Engine::new(db_path).unwrap();
        // 1. Insert 350 items to flush to SSTable 0 (seqs 1..=350)
        for i in 0..350 {
            let k = format!("{:04}", i).into_bytes();
            let v = format!("value{:04}", i).into_bytes();
            engine.set(&k, &v).unwrap();
        }
        // 2. Insert items that remain unflushed in the WAL (seqs 351, 352)
        engine.set(&b"wal_key_1".to_vec(), &b"v1".to_vec()).unwrap();
        engine.set(&b"wal_key_2".to_vec(), &b"v2".to_vec()).unwrap();
    }

    // 3. Reopen engine: must recover both SSTable seq (up to 350) and WAL seq (up to 352)
    let mut reopened = Engine::new(db_path).unwrap();

    // 4. Write new key and flush to SSTable 1
    reopened
        .set(&b"new_after_mixed".to_vec(), &b"mixed_val".to_vec())
        .unwrap();
    for i in 0..350 {
        let k = format!("new_{:04}", i).into_bytes();
        let v = format!("new_value{:04}", i).into_bytes();
        reopened.set(&k, &v).unwrap();
    }

    // 5. Read SSTable 1 to verify sequence is > 352
    let sstable = SstableReader::new(db_path, 1).unwrap();
    let entries: Vec<_> = sstable.iter().unwrap().collect();

    let entry = entries
        .iter()
        .find(|kv| kv.key == b"new_after_mixed".as_slice())
        .expect("new_after_mixed should be in SSTable 1");

    let new_seq = usize::from_le_bytes(entry.sequence);
    assert!(
        new_seq > 352,
        "Expected sequence to be > 352, but got {}",
        new_seq
    );
}
