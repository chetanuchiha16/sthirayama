use crate::memtable::{Memtable, Value};

#[test]
fn test_new_memtable() {
    let memtable = Memtable::new();
    assert_eq!(memtable.size, 0);
}

#[test]
fn test_memtable_insert_and_extract() {
    let mut memtable = Memtable::new();
    let key = b"key1".to_vec();
    let value = b"value1".to_vec();

    memtable.insert(&key, value.clone()).unwrap();

    match memtable.extract(&key).unwrap() {
        Value::Data(extracted_val) => assert_eq!(extracted_val, value),
        _ => panic!("Expected Value::Data"),
    }
}

#[test]
fn test_memtable_extract_missing_key() {
    let memtable = Memtable::new();
    let key = b"missing_key".to_vec();

    match memtable.extract(&key).unwrap() {
        Value::None => (),
        _ => panic!("Expected Value::None for missing key"),
    }
}

#[test]
fn test_memtable_update_value() {
    let mut memtable = Memtable::new();
    let key = b"key1".to_vec();

    memtable.insert(&key, b"initial_val".to_vec()).unwrap();
    memtable.insert(&key, b"updated_val".to_vec()).unwrap();

    match memtable.extract(&key).unwrap() {
        Value::Data(extracted_val) => assert_eq!(extracted_val, b"updated_val".to_vec()),
        _ => panic!("Expected Value::Data with updated value"),
    }
}

#[test]
fn test_memtable_delete() {
    let mut memtable = Memtable::new();
    let key = b"key1".to_vec();
    let value = b"value1".to_vec();

    memtable.insert(&key, value).unwrap();
    memtable.delete(&key);

    match memtable.extract(&key).unwrap() {
        Value::Tombstone => (),
        _ => panic!("Expected Value::Tombstone after delete"),
    }
}

#[test]
fn test_memtable_delete_non_existent_key() {
    let mut memtable = Memtable::new();
    let key = b"non_existent".to_vec();

    memtable.delete(&key);

    match memtable.extract(&key).unwrap() {
        Value::Tombstone => (),
        _ => panic!("Expected Value::Tombstone for deleted non-existent key"),
    }
}

#[test]
fn test_memtable_size_tracking() {
    let mut memtable = Memtable::new();
    assert_eq!(memtable.size, 0);

    let key1 = b"k1".to_vec(); // len 2
    let val1 = b"v1".to_vec(); // len 2
    memtable.insert(&key1, val1).unwrap();
    assert_eq!(memtable.size, 4);

    let key2 = b"key2".to_vec(); // len 4
    let val2 = b"value2".to_vec(); // len 6
    memtable.insert(&key2, val2).unwrap();
    assert_eq!(memtable.size, 14);
}

#[test]
fn test_value_serialization() {
    let data_val = Value::Data(b"hello world".to_vec());
    let data_bytes = data_val.to_bytes();
    match Value::from_bytes(&data_bytes).unwrap() {
        Value::Data(bytes) => assert_eq!(bytes, b"hello world".to_vec()),
        _ => panic!("Failed to deserialize Value::Data"),
    }

    let tombstone_val = Value::Tombstone;
    let tombstone_bytes = tombstone_val.to_bytes();
    match Value::from_bytes(&tombstone_bytes).unwrap() {
        Value::Tombstone => (),
        _ => panic!("Failed to deserialize Value::Tombstone"),
    }

    let none_val = Value::None;
    let none_bytes = none_val.to_bytes();
    match Value::from_bytes(&none_bytes).unwrap() {
        Value::None => (),
        _ => panic!("Failed to deserialize Value::None"),
    }
}
