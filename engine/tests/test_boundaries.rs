use engine::engine::Engine;
use tempfile::tempdir;

#[tokio::test]
async fn test_binary_keys_with_null_and_high_bytes() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();
    let mut engine = Engine::new(db_path, None).unwrap();

    // Keys and values containing embedded null bytes, non-UTF8 binary patterns, and extremes
    let binary_key1 = vec![0x00, 0x00, 0x01, 0x00, 0xFF, 0xFE];
    let binary_val1 = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00];

    let binary_key2 = vec![0x00, 0xFF, 0x00, 0xAA];
    let binary_val2 = vec![0x01, 0x02, 0x03];

    // 1. Memtable test
    engine.set(&binary_key1, &binary_val1).unwrap();
    engine.set(&binary_key2, &binary_val2).unwrap();

    assert_eq!(engine.get(&binary_key1).unwrap(), Some(binary_val1.clone()));
    assert_eq!(engine.get(&binary_key2).unwrap(), Some(binary_val2.clone()));

    // 2. SSTable flush test
    for i in 0..1000 {
        let k = format!("fill_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }

    assert_eq!(
        engine.get(&binary_key1).unwrap(),
        Some(binary_val1.clone()),
        "Binary keys with null bytes must be readable from SSTable"
    );
    assert_eq!(
        engine.get(&binary_key2).unwrap(),
        Some(binary_val2.clone()),
        "Binary keys with null bytes must be readable from SSTable"
    );
}

#[test]
fn test_empty_value_vs_missing_key() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();
    let mut engine = Engine::new(db_path, None).unwrap();

    let empty_val_key = b"key_with_empty_val".to_vec();
    let missing_key = b"key_that_does_not_exist".to_vec();

    // Store empty value (0 bytes)
    engine.set(&empty_val_key, &b"".to_vec()).unwrap();

    // A key with empty value must return Some(vec![])
    let res = engine.get(&empty_val_key).unwrap();
    assert_eq!(
        res,
        Some(vec![]),
        "Key set to empty bytes should return Some([]), not None"
    );

    // A key that was never set must return None
    let missing_res = engine.get(&missing_key).unwrap();
    assert_eq!(
        missing_res,
        None,
        "Missing key must return None"
    );
}

#[tokio::test]
async fn test_large_value_spanning_multiple_blocks() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();
    let mut engine = Engine::new(db_path, None).unwrap();

    let large_key = b"large_payload_key".to_vec();
    // 64 KB payload of repeating deterministic bytes
    let large_value: Vec<u8> = (0..(64 * 1024)).map(|i| (i % 251) as u8).collect();

    engine.set(&large_key, &large_value).unwrap();

    // Must read back cleanly from memtable
    assert_eq!(engine.get(&large_key).unwrap(), Some(large_value.clone()));

    // Flush to SSTable
    for i in 0..1000 {
        let k = format!("fill_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }

    // Must read back cleanly from SSTable
    let retrieved = engine.get(&large_key).unwrap();
    assert_eq!(
        retrieved,
        Some(large_value),
        "Large value (64KB) must be retrieved intact from SSTable"
    );
}

#[tokio::test]
async fn test_frequent_updates_to_single_key() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();
    let mut engine = Engine::new(db_path, None).unwrap();

    let counter_key = b"counter".to_vec();

    // Update same key 50 times, with intermittent flushes
    for i in 0..50 {
        let val = format!("val_{}", i).into_bytes();
        engine.set(&counter_key, &val).unwrap();

        // Every 10 iterations, flush memtable
        if i % 10 == 0 {
            for j in 0..1000 {
                let k = format!("fill_{}_{:04}", i, j).into_bytes();
                let v = format!("val_{:04}", j).into_bytes();
                engine.set(&k, &v).unwrap();
            }
        }
    }

    // Must return the final value (val_49)
    assert_eq!(
        engine.get(&counter_key).unwrap(),
        Some(b"val_49".to_vec()),
        "Latest update must always be returned regardless of flush count"
    );
}
