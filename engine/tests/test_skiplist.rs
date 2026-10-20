use engine::{
    skiplist::{SkipList, SkipListKV, SkipListNode},
    skiplist_error::SkipListError,
};

#[test]
fn test_new_skiplistkv() {
    let skiplist_kv = SkipListKV::new(6, 7, 0usize.to_le_bytes());
    assert_eq!(skiplist_kv.key, 6);
    assert_eq!(skiplist_kv.value, 7);
}

#[test]
fn test_new_skiplist_node() {
    let skiplist_node = SkipListNode::new(5, 6, 7, 0usize.to_le_bytes());
    let key = SkipListNode::get_key(skiplist_node);
    let value = SkipListNode::get_value(skiplist_node);
    assert_eq!(key, &6);
    assert_eq!(value, &7);
}

#[test]
fn test_new_skiplist() -> Result<(), SkipListError> {
    let skiplist = SkipList::new(5, -1, -1);
    let head = skiplist.head;
    let key = SkipListNode::get_key(head);
    let value = SkipListNode::get_value(head);
    assert_eq!(key, &-1);
    assert_eq!(value, &-1);
    assert_eq!(skiplist.max_level, 5);
    Ok(())
}

#[test]
fn test_insert_and_search() -> Result<(), SkipListError> {
    let mut skiplist = SkipList::new(5, -1, -1);
    skiplist.insert(6, 7, 0usize.to_le_bytes());
    assert_eq!(skiplist.search(&6), Some(7));
    assert_eq!(skiplist.search(&7), None);
    Ok(())
}

#[test]
fn test_insert_duplicate_key_does_not_mutate() -> Result<(), SkipListError> {
    let mut skiplist = SkipList::new(5, -1, -1);

    skiplist.insert(6, 7, 0usize.to_le_bytes());
    assert_eq!(skiplist.search(&6), Some(7));

    // In concurrent LSM designs, existing node values are not mutated in-place
    // to eliminate reader-writer data races.
    skiplist.insert(6, 10, 0usize.to_le_bytes());

    assert_eq!(skiplist.search(&6), Some(7));

    Ok(())
}

#[test]
fn test_skiplist_iteration() {
    let mut skiplist = SkipList::new(5, -1, -1);
    skiplist.insert(10, 100, 0usize.to_le_bytes());
    skiplist.insert(5, 50, 0usize.to_le_bytes());
    skiplist.insert(20, 200, 0usize.to_le_bytes());

    let elements: Vec<(i32, i32)> = skiplist.iter().map(|kv| (kv.key, kv.value)).collect();
    assert_eq!(elements, vec![(5, 50), (10, 100), (20, 200)]);
}

#[test]
fn test_load_and_store_forward() {
    let node1 = SkipListNode::new(2, 1, 10, 0usize.to_le_bytes());
    let node2 = SkipListNode::new(2, 2, 20, 0usize.to_le_bytes());

    assert!(SkipListNode::load_forward(node1, 0).is_null());
    SkipListNode::store_forward(node1, 0, node2);
    assert_eq!(SkipListNode::load_forward(node1, 0), node2);
}

#[test]
fn test_concurrent_search() {
    use std::sync::Arc;
    use std::thread;

    let mut skiplist = SkipList::new(8, -1, -1);
    for i in 0..100 {
        skiplist.insert(i, i * 10, 0usize.to_le_bytes());
    }

    let skiplist = Arc::new(skiplist);
    let mut handles = Vec::new();

    for _ in 0..4 {
        let sl = Arc::clone(&skiplist);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                assert_eq!(sl.search(&i), Some(i * 10));
            }
            assert_eq!(sl.search(&200), None);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_skiplist_display() {
    let mut skiplist = SkipList::new(4, -1, -1);
    skiplist.insert(1, 10, 0usize.to_le_bytes());
    skiplist.insert(2, 20, 0usize.to_le_bytes());
    let display_str = format!("{}", skiplist);
    assert!(display_str.contains("SkipList (Height: 4)"));
    assert!(display_str.contains("Level"));
}
