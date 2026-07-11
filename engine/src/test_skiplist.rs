use crate::skiplist::{SkipList, SkipListKV, SkipListNode};

#[test]
fn test_new_skiplistkv() {
    let skiplist_kv = SkipListKV::new(6, 7);
    assert_eq!(skiplist_kv.key, 6);
    assert_eq!(skiplist_kv.value, 7);
}

#[test]
fn test_new_skiplist_node() {
    let skiplist_node = SkipListNode::new(5, 6, 7);
    let key = SkipListNode::get_key(&skiplist_node);
    let value = SkipListNode::get_value(&skiplist_node);
    assert_eq!(key, &6);
    assert_eq!(value, &7);
}

#[test]
fn test_new_skiplist() {
    let skiplist = SkipList::new(5, -1, -1).unwrap();
    let head = skiplist.head.unwrap();
    let key = SkipListNode::get_key(&head);
    let value = SkipListNode::get_value(&head);
    assert_eq!(key, &-1);
    assert_eq!(value, &-1);
    assert_eq!(skiplist.max_level, 5);
}

#[test]
fn test_insert_and_search() {
    let mut skiplist = SkipList::new(5, -1, -1).unwrap();
    skiplist.insert_with_wal(6, 7).unwrap();
    assert_eq!(skiplist.search(6), Some(7));
    assert_eq!(skiplist.search(7), None);
}
