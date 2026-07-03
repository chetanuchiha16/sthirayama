use std::{fmt::Display, ptr::NonNull};

#[derive(Debug)]
pub struct SkipListNode<K, V> {
    pub level: usize,
    pub key: K,
    pub value: V,
    pub forward: Vec<Option<NonNull<SkipListNode<K, V>>>>,
}

impl<K, V> SkipListNode<K, V>
where
    K: PartialOrd + Display,
    V: Clone,
{
    pub fn new(level: usize, key: K, value: V) -> NonNull<Self> {
        let node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Self {
                key: key,
                value,
                forward: (0..=level).map(|_| None).collect(),
                level,
            })))
        };
        node
    }

    pub fn get_key(node: &NonNull<Self>) -> &K {
        unsafe { &node.as_ref().key }
    }

    pub fn get_value(node: &NonNull<Self>) -> &V {
        unsafe { &node.as_ref().value }
    }

    pub fn get_forward(node: &NonNull<Self>) -> &Vec<Option<NonNull<SkipListNode<K, V>>>> {
        unsafe { &node.as_ref().forward }
    }

    pub fn get_forward_mut(
        node: &mut NonNull<Self>,
    ) -> &mut Vec<Option<NonNull<SkipListNode<K, V>>>> {
        unsafe { &mut node.as_mut().forward }
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    pub max_level: usize,
    pub head: Option<NonNull<SkipListNode<K, V>>>,
}

impl<K, V> SkipList<K, V>
where
    K: PartialOrd + Clone + Display,
    V: Clone,
{
    /// create a new skiplist with a sentinel head
    pub fn new(max_level: usize, dummy_k: K, dummy_v: V) -> Self {
        let head = SkipListNode::new(max_level, dummy_k, dummy_v);
        Self {
            max_level,
            head: Some(head),
        }
    }
    /// generate a random level for the node to be inserted with
    pub fn random_level(&self) -> usize {
        fastrand::usize(1..=self.max_level)
    }

    pub fn search(&self, key: K) -> Option<V> {
        let mut current: NonNull<SkipListNode<K, V>> = self.head?; //caused having reference to temp
        for level in (0..self.max_level).rev() {
            while let Some(node) = SkipListNode::get_forward(&current)[level]
                && SkipListNode::get_key(&node) <= &key
            {
                current = node;
            }
        }
        let cur_k = SkipListNode::get_key(&current).to_owned();
        let cur_v = SkipListNode::get_value(&current).to_owned();
        if cur_k == key { Some(cur_v) } else { None }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut update: Vec<NonNull<SkipListNode<K, V>>> = vec![self.head.unwrap(); self.max_level];
        let new_node_level = self.random_level();
        let mut new_node = SkipListNode::new(new_node_level, key.clone(), value);
        let mut current = self.head.unwrap(); //caused having reference to temp
        for level in (0..self.max_level).rev() {
            while let Some(node) = SkipListNode::get_forward(&current)[level]
                && SkipListNode::get_key(&node) < &key
            {
                current = node;
            }
            update[level] = current;
        }

        for level in (0..new_node_level).rev() {
            SkipListNode::get_forward_mut(&mut new_node)[level] =
                SkipListNode::get_forward_mut(&mut update[level])[level];
            SkipListNode::get_forward_mut(&mut update[level])[level] = Some(new_node);
        }
    }
}

impl<K: Display + PartialOrd, V: Clone> Display for SkipList<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        let mut current = self.head.unwrap();
        while let Some(cur_node) = SkipListNode::get_forward(&current)[0] {
            write!(f, "{} ", SkipListNode::get_key(&cur_node))?;
            current = cur_node;
        }
        write!(f, "]")
    }
}
