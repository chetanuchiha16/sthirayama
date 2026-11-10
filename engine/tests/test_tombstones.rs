use engine::engine::Engine;
use tempfile::tempdir;

#[tokio::test]
async fn test_tombstone_resurrection_memtable_over_sstable() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();
    let mut engine = Engine::new(db_path, None).unwrap();

    let target = b"user:100".to_vec();

    // 1. SET v1 and flush to SSTable 0
    engine.set(&target, &b"v1_initial".to_vec()).unwrap();
    for i in 0..1000 {
        let k = format!("fill_0_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }
    assert_eq!(engine.get(&target).unwrap(), Some(b"v1_initial".to_vec()));

    // 2. DEL target and flush tombstone to SSTable 1
    engine.del(&target).unwrap();
    for i in 0..1000 {
        let k = format!("fill_1_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }
    // At this point, key must be None (tombstone active in SSTable 1)
    assert_eq!(engine.get(&target).unwrap(), None);

    // 3. Resurrect target: SET v2 in active MemTable (do NOT flush)
    engine.set(&target, &b"v2_resurrected".to_vec()).unwrap();

    // Latest value in MemTable must win over the tombstone in SSTable 1
    assert_eq!(
        engine.get(&target).unwrap(),
        Some(b"v2_resurrected".to_vec()),
        "Active memtable write must shadow tombstone in older SSTable"
    );
}

#[tokio::test]
async fn test_tombstone_resurrection_across_multiple_flushes() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();
    let mut engine = Engine::new(db_path, None).unwrap();

    let target = b"resurrect_target".to_vec();

    // Flush 1: Set to v1
    engine.set(&target, &b"version_1".to_vec()).unwrap();
    for i in 0..1000 {
        let k = format!("f1_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }

    // Flush 2: Delete target
    engine.del(&target).unwrap();
    for i in 0..1000 {
        let k = format!("f2_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }
    assert_eq!(engine.get(&target).unwrap(), None);

    // Flush 3: Set to v2 and flush to SSTable 2
    engine.set(&target, &b"version_2".to_vec()).unwrap();
    for i in 0..1000 {
        let k = format!("f3_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }

    // SSTable 2 has higher sequence number than SSTable 1 tombstone
    assert_eq!(
        engine.get(&target).unwrap(),
        Some(b"version_2".to_vec()),
        "Flushed SSTable 2 must shadow flushed SSTable 1 tombstone"
    );

    // Flush 4: Delete target again and flush to SSTable 3
    engine.del(&target).unwrap();
    for i in 0..1000 {
        let k = format!("f4_{:04}", i).into_bytes();
        let v = format!("val_{:04}", i).into_bytes();
        engine.set(&k, &v).unwrap();
    }

    assert_eq!(
        engine.get(&target).unwrap(),
        None,
        "Second deletion must shadow version_2"
    );
}

#[tokio::test]
async fn test_delete_survives_reopen_and_subsequent_set() {
    let dir = tempdir().unwrap();
    let db_path = dir.path();

    let target = b"my_persisted_key".to_vec();

    {
        let mut engine = Engine::new(db_path, None).unwrap();
        engine.set(&target, &b"original".to_vec()).unwrap();

        // Flush original to SSTable
        for i in 0..1000 {
            let k = format!("fill_{:04}", i).into_bytes();
            let v = format!("val_{:04}", i).into_bytes();
            engine.set(&k, &v).unwrap();
        }

        // Delete in memtable/WAL (not flushed to SSTable yet)
        engine.del(&target).unwrap();
    }

    // Reopen engine
    {
        let mut reopened = Engine::new(db_path, None).unwrap();
        // Tombstone must be recovered from WAL
        assert_eq!(
            reopened.get(&target).unwrap(),
            None,
            "Tombstone in WAL must be recovered after restart"
        );

        // Now set new value in reopened engine
        reopened.set(&target, &b"brand_new".to_vec()).unwrap();
        assert_eq!(
            reopened.get(&target).unwrap(),
            Some(b"brand_new".to_vec())
        );
    }
}
