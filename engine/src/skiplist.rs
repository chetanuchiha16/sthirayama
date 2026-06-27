
pub struct SkipListNode<K, V> {
    key: K,
    value: V,
    next_nodes: Vec<Option<Box<SkipListNode<K, V>>>>
}

pub struct SkipList<K, V> {
    max_level: usize,
    head: Vec<Option<Box<SkipListNode<K, V>>>>,
}

impl <K, V>SkipList<K, V> {
    fn new(max_level: usize) -> Self {
        Self { max_level: max_level, head:  (0..max_level).map(|_| None).collect() }
    }
}