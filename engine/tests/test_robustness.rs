use std::fs::{self, File};
use engine::engine::Engine;
use tempfile::tempdir;

#[tokio::test]
async fn test_recovery_with_sparse_sstable_gaps() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    {
        let mut engine = Engine::new(db_path, None).unwrap();

        // 1. Fill and flush SSTable 0 (000000.sst)
        engine.set(&b"key_sst_0".to_vec(), &b"val_sst_0".to_vec()).unwrap();
        for i in 0..280 {
            let k = format!("batch0_{:04}", i).into_bytes();
            let v = format!("val_{:04}", i).into_bytes();
            engine.set(&k, &v).unwrap();
        }

        // 2. Fill and flush SSTable 1 (000001.sst)
        engine.set(&b"key_sst_1".to_vec(), &b"val_sst_1".to_vec()).unwrap();
        for i in 0..280 {
            let k = format!("batch1_{:04}", i).into_bytes();
            let v = format!("val_{:04}", i).into_bytes();
            engine.set(&k, &v).unwrap();
        }

        // 3. Fill and flush SSTable 2 (000002.sst)
        engine.set(&b"key_sst_2".to_vec(), &b"val_sst_2".to_vec()).unwrap();
        for i in 0..280 {
            let k = format!("batch2_{:04}", i).into_bytes();
            let v = format!("val_{:04}", i).into_bytes();
            engine.set(&k, &v).unwrap();
        }
    }

    // Verify all 3 SSTables exist
    let sst0 = db_path.join("000000.sst");
    let sst1 = db_path.join("000001.sst");
    let sst2 = db_path.join("000002.sst");
    assert!(sst0.exists(), "000000.sst must exist");
    assert!(sst1.exists(), "000001.sst must exist");
    assert!(sst2.exists(), "000002.sst must exist");

    // Delete the middle file (000001.sst) to create a sparse gap: [0, 2]
    fs::remove_file(&sst1).unwrap();
    assert!(!sst1.exists());

    // Reopen engine: verify it recovers cleanly with the sparse gap
    let reopened = Engine::new(db_path, None).unwrap();

    // Keys in 000000.sst and 000002.sst must still be readable
    assert_eq!(
        reopened.get(&b"key_sst_0".to_vec()).unwrap(),
        Some(b"val_sst_0".to_vec()),
        "key_sst_0 in surviving SSTable 0 must be readable"
    );
    assert_eq!(
        reopened.get(&b"key_sst_2".to_vec()).unwrap(),
        Some(b"val_sst_2".to_vec()),
        "key_sst_2 in surviving SSTable 2 must be readable"
    );

    // Key in deleted SSTable 1 must be None
    assert_eq!(
        reopened.get(&b"key_sst_1".to_vec()).unwrap(),
        None,
        "key_sst_1 in deleted SSTable 1 must return None"
    );
}

#[tokio::test]
async fn test_recovery_ignores_stray_non_sst_files() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    {
        let mut engine = Engine::new(db_path, Some(1024)).unwrap();
        engine.set(&b"valid_key".to_vec(), &b"valid_val".to_vec()).unwrap();
        for i in 0..500 {
            let k = format!("batch_{:04}", i).into_bytes();
            let v = format!("val_{:04}", i).into_bytes();
            engine.set(&k, &v).unwrap();
        }
    }

    // Add stray temporary and non-sst files in the storage directory
    File::create(db_path.join("000001.tmp")).unwrap();
    File::create(db_path.join("compaction.backup")).unwrap();
    File::create(db_path.join("stray_notes.txt")).unwrap();
    File::create(db_path.join(".hidden_file")).unwrap();

    // Reopen engine: should ignore stray files and boot cleanly
    let reopened = Engine::new(db_path, Some(1024)).unwrap();
    assert_eq!(
        reopened.get(&b"valid_key".to_vec()).unwrap(),
        Some(b"valid_val".to_vec())
    );
}

#[tokio::test]
async fn test_recovery_with_truncated_wal() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    {
        let mut engine = Engine::new(db_path, None).unwrap();
        // Insert a few keys without exceeding memtable (all live in WAL)
        engine.set(&b"wal_key_1".to_vec(), &b"val_1".to_vec()).unwrap();
        engine.set(&b"wal_key_2".to_vec(), &b"val_2".to_vec()).unwrap();
        engine.set(&b"wal_key_3".to_vec(), &b"val_3".to_vec()).unwrap();
    }

    // Truncate the WAL by 4 bytes (simulating a crash mid-write on the 3rd record)
    let wal_path = db_path.join("wal").join("file.wal");
    let mut wal_bytes = fs::read(&wal_path).unwrap();
    assert!(wal_bytes.len() > 10, "WAL should contain data");
    wal_bytes.truncate(wal_bytes.len() - 4);
    fs::write(&wal_path, wal_bytes).unwrap();

    // Reopen engine: verify it does not panic on the truncated tail
    let result = Engine::new(db_path, None);
    match result {
        Ok(reopened) => {
            // If it succeeds, the first record must be intact
            assert_eq!(
                reopened.get(&b"wal_key_1".to_vec()).unwrap(),
                Some(b"val_1".to_vec())
            );
        }
        Err(err) => {
            // Graceful error returned
            println!("Engine returned graceful error on corrupted WAL: {:?}", err);
        }
    }
}
