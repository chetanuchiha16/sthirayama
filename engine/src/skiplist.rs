use std::ptr::NonNull;

#[derive(Debug)]
pub struct SkipListNode<K, V> {
    pub level: usize,
    pub key: K,
    pub value: V,
    pub forward: Vec<Option<NonNull<SkipListNode<K, V>>>>,
}

impl<K, V> SkipListNode<K, V>
where
    K: PartialOrd,
    V: Clone,
{
    pub fn new(level: usize, key: K, value: V) -> Option<NonNull<Self>> {
        let node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Self {
                key: key,
                value,
                forward: (0..=level).map(|_| None).collect(),
                level,
            })))
        };
        Some(node)
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    pub max_level: usize,
    pub head: Option<NonNull<SkipListNode<K, V>>>,
}

impl<K, V> SkipList<K, V>
where
    K: PartialOrd + Clone,
    V: Clone,
{
    /// create a new skiplist with a sentinel head
    pub fn new(max_level: usize, dummy_k: K, dummy_v: V) -> Self {
        let head = SkipListNode::new(max_level, dummy_k, dummy_v);
        Self { max_level, head }
    }
    /// generate a random level for the node to be inserted with
    pub fn random_level(&self) -> usize {
        fastrand::usize(1..=self.max_level)
    }

    pub fn search(&self, key: K) -> Option<V> {
        let mut current = self.head?; //caused having reference to temp
        for level in (0..self.max_level).rev() {
            while let Some(node) = unsafe { current.as_ref().forward[level] }
                && unsafe { &node.as_ref().key } <= &key
            {
                current = node;
            }
        }
        Some(unsafe { current.as_ref().value.to_owned() })
    }

    pub fn insert(&self, key: K, value: V) {
        let new_node_level = self.random_level();
        let new_node = SkipListNode::new(new_node_level, key.clone(), value);
        let mut current = self.head.unwrap(); //caused having reference to temp
        for level in (0..self.max_level).rev() {
            while let Some(node) = unsafe { current.as_ref().forward[level] }
                && unsafe { &node.as_ref().key } < &key
            {
                current = node;
            }
        }
        unsafe { current.as_mut().forward[new_node_level] = new_node };
    }
}
