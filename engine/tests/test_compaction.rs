use tempfile::tempdir;

use engine::{
    compaction::Compaction,
    engine::Engine,
    engine_error::EngineError,
    memtable::{Memtable, Value},
    sstable::{reader::SstableReader, writer::SstableWriter},
};

#[test]
fn test_merge_iterators_deduplication_and_order() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let path = dir.path();

    // SSTable 0: key "a" (seq 1), key "c" (seq 2)
    {
        let mut mem0 = Memtable::new();
        mem0.insert(&b"a".to_vec(), b"val_a_v1".to_vec(), 1)?;
        mem0.insert(&b"c".to_vec(), b"val_c_v1".to_vec(), 2)?;
        let mut writer0 = SstableWriter::new(path, 0)?;
        writer0.write(mem0.skiplist.iter())?;
    }

    // SSTable 1: key "a" (seq 5 - newer!), key "b" (seq 3)
    {
        let mut mem1 = Memtable::new();
        mem1.insert(&b"a".to_vec(), b"val_a_v2".to_vec(), 5)?;
        mem1.insert(&b"b".to_vec(), b"val_b_v1".to_vec(), 3)?;
        let mut writer1 = SstableWriter::new(path, 1)?;
        writer1.write(mem1.skiplist.iter())?;
    }

    // SSTable 2: key "c" (seq 8 - newer!), key "d" (seq 4)
    {
        let mut mem2 = Memtable::new();
        mem2.insert(&b"c".to_vec(), b"val_c_v2".to_vec(), 8)?;
        mem2.insert(&b"d".to_vec(), b"val_d_v1".to_vec(), 4)?;
        let mut writer2 = SstableWriter::new(path, 2)?;
        writer2.write(mem2.skiplist.iter())?;
    }

    let r0 = SstableReader::new(path, 0)?.iter()?;
    let r1 = SstableReader::new(path, 1)?.iter()?;
    let r2 = SstableReader::new(path, 2)?.iter()?;

    let merged: Vec<_> = Compaction::merge_iterators(vec![r0, r1, r2]).collect();

    // Expected keys: "a", "b", "c", "d" in strictly ascending sorted order
    let keys: Vec<_> = merged.iter().map(|kv| kv.key.clone()).collect();
    assert_eq!(
        keys,
        vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec(), b"d".to_vec()]
    );

    // Verify latest values won:
    // "a" should have val_a_v2 (seq 5)
    let val_a = Value::from_bytes(&merged[0].value).unwrap();
    assert_eq!(val_a, Value::Data(b"val_a_v2".to_vec()));

    // "b" should have val_b_v1 (seq 3)
    let val_b = Value::from_bytes(&merged[1].value).unwrap();
    assert_eq!(val_b, Value::Data(b"val_b_v1".to_vec()));

    // "c" should have val_c_v2 (seq 8)
    let val_c = Value::from_bytes(&merged[2].value).unwrap();
    assert_eq!(val_c, Value::Data(b"val_c_v2".to_vec()));

    // "d" should have val_d_v1 (seq 4)
    let val_d = Value::from_bytes(&merged[3].value).unwrap();
    assert_eq!(val_d, Value::Data(b"val_d_v1".to_vec()));

    Ok(())
}

#[tokio::test]
async fn test_compaction_guard_less_than_threshold() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let mut engine = Engine::new(dir.path())?;

    // Calling compact with fewer than 5 tables should be a harmless no-op
    engine.compact()?;

    // Insert 100 entries (not enough to exceed 4 tables)
    for i in 0..100 {
        let k = format!("key_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v)?;
    }

    engine.compact()?;

    // All keys should still be readable
    for i in 0..100 {
        let k = format!("key_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        assert_eq!(engine.get(&k)?, Some(v));
    }

    Ok(())
}

#[tokio::test]
async fn test_engine_compaction_deletes_old_files_and_preserves_data() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let db_path = dir.path();
    let mut engine = Engine::new(db_path)?;

    // Insert enough data to trigger multiple flushes and compaction (> 4 SSTables)
    for i in 0..6000 {
        let k = format!("{:04}", i).into_bytes();
        let v = format!("value{:04}", i).into_bytes();
        engine.set(&k, &v)?;
    }

    // Verify all keys are retrievable across compacted and uncompacted SSTables
    for i in 0..6000 {
        let k = format!("{:04}", i).into_bytes();
        let expected = format!("value{:04}", i).into_bytes();
        assert_eq!(
            engine.get(&k)?,
            Some(expected),
            "Failed reading key {:04} after compaction",
            i
        );
    }

    // The first 4 tables (000000.sst .. 000003.sst) should have been deleted from disk
    assert!(
        !db_path.join("000000.sst").exists(),
        "000000.sst should have been deleted after compaction"
    );
    assert!(
        !db_path.join("000001.sst").exists(),
        "000001.sst should have been deleted after compaction"
    );
    assert!(
        !db_path.join("000002.sst").exists(),
        "000002.sst should have been deleted after compaction"
    );
    assert!(
        !db_path.join("000003.sst").exists(),
        "000003.sst should have been deleted after compaction"
    );

    Ok(())
}

#[tokio::test]
async fn test_engine_compaction_with_updates() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let mut engine = Engine::new(dir.path())?;

    let target_key = b"target_key".to_vec();

    // Set target_key to v1 in early batch
    engine.set(&target_key, &b"v1".to_vec())?;
    for i in 0..1000 {
        let k = format!("dummy_a_{:04}", i).into_bytes();
        engine.set(&k, &b"dummy".to_vec())?;
    }

    // Update target_key to v2 in later batch
    engine.set(&target_key, &b"v2".to_vec())?;
    for i in 0..5000 {
        let k = format!("dummy_b_{:04}", i).into_bytes();
        engine.set(&k, &b"dummy".to_vec())?;
    }

    // Verify the latest value survives compaction
    assert_eq!(engine.get(&target_key)?, Some(b"v2".to_vec()));

    Ok(())
}

#[tokio::test]
async fn test_engine_compaction_with_delete() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let mut engine = Engine::new(dir.path())?;

    let target_key = b"delete_target".to_vec();

    // Insert target key
    engine.set(&target_key, &b"initial_value".to_vec())?;
    for i in 0..1000 {
        let k = format!("dummy_a_{:04}", i).into_bytes();
        engine.set(&k, &b"dummy".to_vec())?;
    }

    // Delete target key
    engine.del(&target_key)?;

    // Add more entries to trigger multiple flushes and compaction
    for i in 0..5000 {
        let k = format!("dummy_b_{:04}", i).into_bytes();
        engine.set(&k, &b"dummy".to_vec())?;
    }

    // The key should remain deleted after compaction
    assert_eq!(engine.get(&target_key)?, None);

    Ok(())
}

#[tokio::test]
async fn test_compaction_reopen_persistence() -> Result<(), EngineError> {
    let dir = tempdir()?;
    let db_path = dir.path();

    {
        let mut engine = Engine::new(db_path)?;
        for i in 0..6000 {
            let k = format!("{:04}", i).into_bytes();
            let v = format!("v_{:04}", i).into_bytes();
            engine.set(&k, &v)?;
        }
    }

    // Reopen the engine from disk
    let mut reopened = Engine::new(db_path)?;
    for i in 0..6000 {
        let k = format!("{:04}", i).into_bytes();
        let expected = format!("v_{:04}", i).into_bytes();
        assert_eq!(
            reopened.get(&k)?,
            Some(expected),
            "Failed reading key {:04} after reopen",
            i
        );
    }

    Ok(())
}
