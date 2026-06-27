#[derive(Debug)]
pub struct SkipListNode<K, V> {
    pub level: usize,
    pub key: K,
    pub value: V,
    pub next_nodes: Vec<Option<Box<SkipListNode<K, V>>>>,
}

impl<K, V> SkipListNode<K, V> {
    pub fn new(level: usize, key: K, value: V) -> Self {
        Self {
            key,
            value,
            next_nodes: (0..level).map(|_| None).collect(),
            level,
        }
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    pub max_level: usize,
    pub head: Vec<Option<Box<SkipListNode<K, V>>>>,
}

impl<K, V> SkipList<K, V> {
    pub fn new(max_level: usize) -> Self {
        Self {
            max_level: max_level,
            head: (0..max_level).map(|_| None).collect(),
        }
    }
}
