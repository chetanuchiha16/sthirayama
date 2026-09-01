use crate::{
    memtable::{Memtable, Value},
    wal::Wal,
};

#[test]
fn test_wal_append_and_recover() {
    let _ = std::fs::create_dir_all("wal");
    let tmpfile = tempfile::NamedTempFile::new().unwrap();
    let path = tmpfile.path();
    let mut wal = Wal::new(path.parent().unwrap()).unwrap();
    let _ = wal.recycle();

    let key1 = b"wal_key1".to_vec();
    let val1 = Value::Data(b"wal_val1".to_vec());
    wal.append(&key1, val1).unwrap();

    let key2 = b"wal_key2".to_vec();
    let val2 = Value::Tombstone;
    wal.append(&key2, val2).unwrap();

    let mut memtable = Memtable::new();
    wal.recover::<Vec<u8>, Vec<u8>>(&mut memtable.skiplist)
        .unwrap();

    match memtable.extract(&key1).unwrap() {
        Value::Data(v) => assert_eq!(v, b"wal_val1".to_vec()),
        _ => panic!("Expected Value::Data for wal_key1"),
    }

    match memtable.extract(&key2).unwrap() {
        Value::Tombstone => (),
        _ => panic!("Expected Value::Tombstone for wal_key2"),
    }
}

#[test]
fn test_wal_recycle() {
    let _ = std::fs::create_dir_all("wal");
    let tmpfile = tempfile::NamedTempFile::new().unwrap();
    let path = tmpfile.path();
    let mut wal = Wal::new(path.parent().unwrap()).unwrap();
    let _ = wal.recycle();

    let key = b"recycle_key".to_vec();
    let val = Value::Data(b"recycle_val".to_vec());
    wal.append(&key, val).unwrap();

    // Recycle clears existing log
    wal.recycle().unwrap();

    let mut memtable = Memtable::new();
    wal.recover::<Vec<u8>, Vec<u8>>(&mut memtable.skiplist)
        .unwrap();

    match memtable.extract(&key).unwrap() {
        Value::None => (),
        _ => panic!("Expected Value::None after WAL recycling"),
    }
}
