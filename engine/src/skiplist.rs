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
    K: Clone + PartialOrd,
    V: Clone,
{
    pub fn new(level: usize, key: &K, value: V) -> Self {
        Self {
            key: key.to_owned(),
            value,
            forward: (0..level).map(|_| None).collect(),
            level,
        }
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    pub max_level: usize,
    pub head: Option<NonNull<SkipListNode<K, V>>>,
}

impl<K, V> SkipList<K, V>
where
    K: Clone + PartialOrd,
    V: Clone,
{
    /// create a new skiplist with a sentinel head
    pub fn new(max_level: usize, dummy_k: K, dummy_v: V) -> Self {
        let head = Box::new(SkipListNode::new(max_level, &dummy_k, dummy_v));
        let head_ptr: *mut SkipListNode<K, V> = Box::into_raw(head);

        Self {
            max_level,
            head: Some(unsafe { NonNull::new_unchecked(head_ptr) }),
        }
    }

    pub fn random_level(&self) -> usize {
        fastrand::usize(0..self.max_level)
    }

    pub fn search(&self, key: K) -> Option<V> {
        let mut current = self.head?; //caused having reference to temp
        for level in (0..self.max_level).rev() {
            while let Some(node) = unsafe { current.as_ref().forward[level] }
                && unsafe { &node.as_ref().key } < &key
            {
                current = node;
            }
        }
        Some(unsafe { current.as_ref().value.to_owned() })
    }
}
