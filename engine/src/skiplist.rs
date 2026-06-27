#[derive(Debug)]
pub struct SkipListNode<K, V> {
    pub level: usize,
    pub key: K,
    pub value: V,
    pub next_nodes: Vec<Option<usize>>,
}

impl<K, V> SkipListNode<K, V>
where
    K: Clone,
{
    pub fn new(level: usize, key: &K, value: V) -> Self {
        Self {
            key: key.to_owned(),
            value,
            next_nodes: (0..level).map(|_| None).collect(),
            level,
        }
    }
}

#[derive(Debug)]
pub struct SkipList {
    pub max_level: usize,
    pub head: Vec<Option<usize>>,
}

impl SkipList {
    pub fn new(max_level: usize) -> Self {
        Self {
            max_level,
            head: (0..max_level).map(|_| None).collect(),
        }
    }

    fn random_level(&self) -> usize {
        fastrand::usize(0..self.max_level)
    }
}
