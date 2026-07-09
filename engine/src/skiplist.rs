use std::fmt::{Display, Formatter};
use std::io::Error;
use std::ptr::NonNull;

use bitcode::{Decode, Encode};

use crate::wal::Wal;

#[derive(Debug, Encode, Decode, Clone)]
pub struct SkipListKV<K, V> {
    pub key: K,
    pub value: V,
}

impl<K: Clone, V: Clone> SkipListKV<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

#[derive(Debug)]
pub struct SkipListNode<K, V> {
    pub level: usize,
    pub data: SkipListKV<K, V>,
    pub forward: Vec<Option<NonNull<SkipListNode<K, V>>>>,
}

impl<K, V> SkipListNode<K, V>
where
    K: PartialOrd + Display + Clone,
    V: Clone + Display,
{
    pub fn new(level: usize, key: K, value: V) -> NonNull<Self> {
        let node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Self {
                data: SkipListKV::new(key, value),
                forward: (0..=level).map(|_| None).collect(),
                level,
            })))
        };
        node
    }

    pub fn get_key(node: &NonNull<Self>) -> &K {
        unsafe { &node.as_ref().data.key }
    }

    pub fn get_value(node: &NonNull<Self>) -> &V {
        unsafe { &node.as_ref().data.value }
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
    pub wal: Wal,
}

impl<K, V> SkipList<K, V>
where
    K: PartialOrd + Clone + Display + Encode,
    V: Clone + Display + Encode,
{
    /// create a new skiplist with a sentinel head
    pub fn new(max_level: usize, dummy_k: K, dummy_v: V) -> Result<Self, Error> {
        let head = SkipListNode::new(max_level, dummy_k.clone(), dummy_v.clone());
        let wal = Wal::new()?;
        Ok(Self {
            max_level,
            head: Some(head),
            wal,
        })
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

    pub fn insert_with_wal(&mut self, key: K, value: V) -> Result<(), std::io::Error> {
        self.wal.append(key.clone(), value.clone())?;
        self.insert(key, value)?;
        Ok(())
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<(), std::io::Error> {
        let data = SkipListKV::new(key, value);
        let new_node_level = self.random_level();
        let mut new_node = SkipListNode::new(new_node_level, data.key.clone(), data.value);
        let mut update: Vec<NonNull<SkipListNode<K, V>>> = vec![self.head.unwrap(); self.max_level];
        let mut current = self.head.unwrap(); //caused having reference to temp
        for level in (0..self.max_level).rev() {
            while let Some(node) = SkipListNode::get_forward(&current)[level]
                && SkipListNode::get_key(&node) < &data.key
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
        Ok(())
    }
}

impl<K: Display + PartialOrd + Clone, V: Clone + Display> Display for SkipList<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- SkipList (Height: {}) ---", self.max_level)?;

        // --- Step 1: Collect all keys in order from Level 0 to build our columns ---
        let mut columns = Vec::new();
        let mut current = self.head;

        while let Some(cur_node) = current {
            if let Some(next_node) = SkipListNode::get_forward(&cur_node)[0] {
                let key_str = format!("{}", SkipListNode::get_key(&next_node));
                columns.push((next_node.clone(), key_str));
                current = Some(next_node);
            } else {
                break;
            }
        }

        // --- Step 2: Print level by level from top to bottom ---
        for level in (0..self.max_level).rev() {
            write!(f, "Level {:2}: Head", level)?;

            // Start traversal tracking for the current level
            let mut current_node_ptr = self.head;

            // Inside your level loop...
            for (node, key_str) in &columns {
                let next_at_level = current_node_ptr
                    .as_ref()
                    .and_then(|n| SkipListNode::get_forward(n)[level].clone());

                if let Some(ref target) = next_at_level {
                    if SkipListNode::get_key(target) == SkipListNode::get_key(node) {
                        write!(f, " -> [{}]", key_str)?;
                        current_node_ptr = next_at_level;
                        continue;
                    }
                }

                // FIX: Format the structural gap as an empty slot " -> [---]"
                // instead of appending loose hyphens to the previous arrow.
                let dashes = "-".repeat(key_str.len());
                write!(f, " -> [{}]", dashes)?;
            }

            writeln!(f, " -> Nil")?;
        }

        write!(f, "-----------------------------")
    }
}
