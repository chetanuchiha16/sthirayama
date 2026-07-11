use crate::skiplist::SkipListKV;

#[test]
fn test_new_skiplistkv() {
    let skiplist_kv = SkipListKV::new(6, 7);
    assert_eq!(skiplist_kv.key, 6);
    assert_eq!(skiplist_kv.value, 7);
}