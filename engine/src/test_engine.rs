use crate::engine::Engine;

#[test]
fn test_set_and_get_from_memtable() {
    let mut engine = Engine::new("test_set_and_get_from_memtable").unwrap();

    let key = b"0001".to_vec();
    let value = b"value1".to_vec();

    engine.set(&key, value.clone()).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(value));
}

#[test]
fn test_get_missing_key() {
    let mut engine = Engine::new("test_get_missing_key").unwrap();

    let key = b"9999".to_vec();

    assert_eq!(engine.get(&key).unwrap(), None);
}

// #[test]
// fn test_set_overwrites_value() {
//     let mut engine = Engine::new().unwrap();

//     let key = b"0001".to_vec();

//     engine.set(&key, b"first".to_vec()).unwrap();
//     engine.set(&key, b"second".to_vec()).unwrap();

//     assert_eq!(
//         engine.get(&key).unwrap(),
//         Some(b"second".to_vec())
//     );
// }

#[test]
fn test_flush_and_get_from_sstable() {
    let mut engine = Engine::new("test_flush_and_get_from_sstable").unwrap();

    // Make the memtable exceed 4 KB.
    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, value).unwrap();
    }

    // These should now be in an SSTable.
    let key = b"0000".to_vec();
    assert_eq!(engine.get(&key).unwrap(), Some(b"value0000".to_vec()));

    let key = b"0500".to_vec();
    assert_eq!(engine.get(&key).unwrap(), Some(b"value0500".to_vec()));

    let key = b"0999".to_vec();
    assert_eq!(engine.get(&key).unwrap(), Some(b"value0999".to_vec()));
}

#[test]
fn test_flush_multiple_sstables() {
    let mut engine = Engine::new("test_flush_multiple_sstables").unwrap();

    // Enough data to create multiple SSTables.
    for i in 0..5000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, value).unwrap();
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

#[test]
fn test_key_not_found_after_flush() {
    let mut engine = Engine::new("test_key_not_found_after_flush").unwrap();

    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, value).unwrap();
    }

    assert_eq!(engine.get(&b"9999".to_vec()).unwrap(), None);
}

#[test]
fn test_keys_across_multiple_flushes() {
    let mut engine = Engine::new("test_keys_across_multiple_flushes").unwrap();

    for i in 0..10000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("{:04}", i * 2).into_bytes();

        engine.set(&key, value).unwrap();
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
    let mut engine = Engine::new("test_set_overwrites_value").unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, b"first".to_vec()).unwrap();
    assert_eq!(engine.get(&key).unwrap(), Some(b"first".to_vec()));

    // Update the existing key.
    engine.set(&key, b"second".to_vec()).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(b"second".to_vec()));
}

#[test]
fn test_update_after_flush() {
    let mut engine = Engine::new("test_update_after_flush").unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, b"first".to_vec()).unwrap();

    // Force the original value into an SSTable.
    for i in 0..1000 {
        let key = format!("{:04}", i + 10).into_bytes();
        let value = format!("value{:04}", i + 10).into_bytes();
        engine.set(&key, value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"first".to_vec()));

    // New value goes into the newer memtable.
    engine.set(&key, b"second".to_vec()).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(b"second".to_vec()));
}

#[test]
fn test_delete_from_memtable() {
    let mut engine = Engine::new("test_delete_from_memtable").unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, b"value1".to_vec()).unwrap();

    assert_eq!(engine.get(&key).unwrap(), Some(b"value1".to_vec()));

    engine.del(&key);

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[test]
fn test_delete_after_flush() {
    let mut engine = Engine::new("test_delete_after_flush").unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, b"value1".to_vec()).unwrap();

    // Force the value into an SSTable.
    for i in 0..1000 {
        let key = format!("{:04}", i + 10).into_bytes();
        let value = format!("value{:04}", i + 10).into_bytes();

        engine.set(&key, value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"value1".to_vec()));

    // Tombstone goes into the newer MemTable.
    engine.del(&key);

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[test]
fn test_delete_survives_flush() {
    let mut engine = Engine::new("test_delete_survives_flush").unwrap();

    let key = b"0001".to_vec();

    engine.set(&key, b"value1".to_vec()).unwrap();

    // Flush original value.
    for i in 0..1000 {
        let key = format!("{:04}", i + 10).into_bytes();
        let value = format!("value{:04}", i + 10).into_bytes();

        engine.set(&key, value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"value1".to_vec()));

    // Write tombstone.
    engine.del(&key);

    // Force tombstone into an SSTable too.
    for i in 0..1000 {
        let key = format!("{:04}", i + 2000).into_bytes();
        let value = format!("value{:04}", i + 2000).into_bytes();

        engine.set(&key, value).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[test]
fn test_delete_missing_key() {
    let mut engine = Engine::new("test_delete_missing_key").unwrap();

    let key = b"9999".to_vec();

    assert_eq!(engine.get(&key).unwrap(), None);

    engine.del(&key);

    assert_eq!(engine.get(&key).unwrap(), None);
}

#[test]
fn test_overwrite_across_multiple_flushes() {
    let mut engine = Engine::new("test_overwrite_across_multiple_flushes").unwrap();

    let key = b"0001".to_vec();

    // Set key to "first_value" and flush to SSTable 0.
    engine.set(&key, b"first_value".to_vec()).unwrap();
    for i in 0..1000 {
        let k = format!("{:04}", i + 10).into_bytes();
        let v = format!("value{:04}", i + 10).into_bytes();
        engine.set(&k, v).unwrap();
    }

    assert_eq!(engine.get(&key).unwrap(), Some(b"first_value".to_vec()));

    // Overwrite key with "second_value" and flush to SSTable 1.
    engine.set(&key, b"second_value".to_vec()).unwrap();
    for i in 0..1000 {
        let k = format!("{:04}", i + 2000).into_bytes();
        let v = format!("value{:04}", i + 2000).into_bytes();
        engine.set(&k, v).unwrap();
    }

    // Newer SSTable value should override older SSTable value.
    assert_eq!(engine.get(&key).unwrap(), Some(b"second_value".to_vec()));
}

