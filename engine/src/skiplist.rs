use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ptr::{self};
use std::sync::atomic::{AtomicPtr, Ordering};

use bitcode::{Decode, Encode};

use crate::traits::{SkipListIterator, TypeSkipListKey, TypeSkipListValue};

#[derive(Debug, Encode, Decode, Clone)]
pub struct SkipListKV<K, V> {
    pub key: K,
    pub value: V,
    pub sequence: [u8; 8],
}

impl<K: TypeSkipListKey, V: TypeSkipListValue> SkipListKV<K, V> {
    pub fn new(key: K, value: V, sequence: [u8; 8]) -> Self {
        // Self { key, value }
        Self {
            key,
            value,
            sequence: sequence,
        }
    }

    pub fn encode(&self) -> ([u8; 8], Vec<u8>) {
        let kv_as_bytes = bitcode::encode(self);
        let kv_bytes_len_as_bytes = kv_as_bytes.len().to_le_bytes();
        (kv_bytes_len_as_bytes, kv_as_bytes)
    }

    // pub fn to_string(&self) -> String {
    //     let
    // }
}

#[derive(Debug)]
pub struct SkipListNode<K, V> {
    pub level: usize,
    pub data: SkipListKV<K, V>,
    pub forward: Box<[AtomicPtr<SkipListNode<K, V>>]>,
}

impl<K, V> SkipListNode<K, V>
where
    K: TypeSkipListKey,
    V: TypeSkipListValue,
{
    pub fn new(level: usize, key: K, value: V, sequence: [u8; 8]) -> *mut Self {
        let node = Box::into_raw(Box::new(Self {
            data: SkipListKV::new(key, value, sequence),
            forward: (0..=level)
                .map(|_| AtomicPtr::new(ptr::null_mut()))
                .collect(),
            level,
        }));
        node
    }
}
impl<K, V> SkipListNode<K, V> {
    pub fn get_data<'a>(node: *const Self) -> &'a SkipListKV<K, V> {
        unsafe { &(*node).data }
    }

    pub fn get_key<'a>(node: *const Self) -> &'a K {
        unsafe { &(*node).data.key }
    }

    pub fn get_value<'a>(node: *const Self) -> &'a V {
        unsafe { &(*node).data.value }
    }
    // pub fn get_value_mut(node: &mut *const Self) -> &mut V {
    //     unsafe { &mut node.as_mut().data.value }
    // }

    // pub fn get_forward(node: *const Self) -> &Vec<Option<NonNull<SkipListNode<K, V>>>> {
    //     unsafe { &(*node).forward }
    // }

    // pub fn get_forward_mut(
    //     node: &mut NonNull<Self>,
    // ) -> &mut Vec<Option<NonNull<SkipListNode<K, V>>>> {
    //     unsafe { &mut node.as_mut().forward }
    // }
    pub fn load_forward(node: *const Self, level: usize) -> *mut Self {
        unsafe { (*node).forward[level].load(Ordering::Acquire) }
    }

    pub fn store_forward(node: *const Self, level: usize, next: *mut Self) {
        unsafe { (*node).forward[level].store(next, Ordering::Release) }
    }

    pub fn get_sequence(node: *const Self) -> u64 {
        unsafe { u64::from_le_bytes((*node).data.sequence) }
    }

    pub fn compare(node: *const Self, target_key: &K, target_seq: u64) -> std::cmp::Ordering
    where
        K: PartialOrd,
    {
        let node_key = Self::get_key(node);
        match node_key.partial_cmp(target_key) {
            Some(std::cmp::Ordering::Equal) => {
                let node_seq = Self::get_sequence(node);
                node_seq.cmp(&target_seq)
            }
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        }
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    pub max_level: usize,
    pub head: *mut SkipListNode<K, V>,
    // pub wal: Wal,
}
// Safety: Node pointers are published via Release/Acquire atomic barriers.
// Nodes remain immutable after publication.
unsafe impl<K: Send + Sync, V: Send + Sync> Send for SkipList<K, V> {}
unsafe impl<K: Send + Sync, V: Send + Sync> Sync for SkipList<K, V> {}

impl<K, V> SkipList<K, V>
where
    K: TypeSkipListKey,
    V: TypeSkipListValue,
{
    /// create a new skiplist with a sentinel head
    pub fn new(max_level: usize, dummy_k: K, dummy_v: V) -> Self {
        let head = SkipListNode::new(
            max_level,
            dummy_k.clone(),
            dummy_v.clone(),
            0usize.to_le_bytes(),
        );
        Self {
            max_level,
            head: head,
            // wal,
        }
    }

    // pub fn iter(&self) -> Vec<SkipListKV<K, V>> {

    // pub fn iter(&self) -> impl IntoIterator<Item = SkipListKV<K, V>> {
    //     let mut kv_list = Vec::new();
    //     let head = &self.head.unwrap();
    //     let mut current = SkipListNode::get_forward(head)[0];
    //     while let Some(cur_node) = current {
    //         let kv = SkipListNode::get_data(&cur_node).clone();
    //         kv_list.push(kv);
    //         let next_node = SkipListNode::get_forward(&cur_node)[0];
    //         current = next_node;
    //     }
    //     kv_list
    // }

    pub fn iter<'a>(&self) -> SkipListIterator<'a, K, V> {
        let first = SkipListNode::load_forward(self.head, 0);

        SkipListIterator {
            next_node: first,
            marker: PhantomData,
        }
    }

    /// generate a random level for the node to be inserted with
    pub fn random_level(&self) -> usize {
        fastrand::usize(1..=self.max_level)
    }
    /// search the skiplist for key
    pub fn search(&self, key: &K) -> Option<V> {
        let mut current = self.head; //caused having reference to temp
        for level in (0..self.max_level).rev() {
            loop {
                let next = SkipListNode::load_forward(current, level);
                if next.is_null() {
                    break;
                }
                if SkipListNode::compare(next, key, u64::MAX).is_le() {
                    current = next;
                } else {
                    break;
                }
            }
        }
        let cur_k = SkipListNode::get_key(current);
        let cur_v = SkipListNode::get_value(current);
        if current != self.head && cur_k == key {
            Some(cur_v.to_owned())
        } else {
            None
        }
    }

    /// Insert requires &mut self (single-writer model).
    /// Readers can execute search() concurrently without blocking.
    pub fn insert(&self, key: K, value: V, sequence: [u8; 8]) {
        let target_seq = u64::from_le_bytes(sequence);
        let mut update: Vec<*mut SkipListNode<K, V>> = vec![self.head; self.max_level];
        let mut current = self.head;

        for level in (0..self.max_level).rev() {
            loop {
                let next = SkipListNode::load_forward(current, level);
                if next.is_null() {
                    break;
                }
                if SkipListNode::compare(next, &key, target_seq).is_le() {
                    current = next;
                } else {
                    break;
                }
            }
            update[level] = current;
        }

        // In concurrent LSM designs, avoid mutating existing node values in-place
        // to eliminate reader-writer data races. Replace the node or track versions.

        let new_node_level = self.random_level();
        let new_node = SkipListNode::new(new_node_level, key, value, sequence);

        // Step 1: Initialize new node pointers before publication.
        // Relaxed ordering is sufficient here because new_node is not yet visible to readers.
        for level in 0..new_node_level {
            let next = SkipListNode::load_forward(update[level], level);
            unsafe {
                (*new_node).forward[level].store(next, Ordering::Relaxed);
            }
        }

        // Step 2: Publish new node to readers from level 0 upwards using Release ordering.
        for level in 0..new_node_level {
            SkipListNode::store_forward(update[level], level, new_node);
        }
    }
}

impl<K, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        let mut curr = self.head;
        while !curr.is_null() {
            let next = SkipListNode::load_forward(curr, 0);
            unsafe {
                drop(Box::from_raw(curr));
            }
            curr = next;
        }
    }
}

impl<K: TypeSkipListKey, V: TypeSkipListValue> Display for SkipList<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- SkipList (Height: {}) ---", self.max_level)?;

        let mut columns = Vec::new();
        let mut current = SkipListNode::load_forward(self.head, 0);

        while !current.is_null() {
            let key_str = format!("{:?}", SkipListNode::get_key(current));
            columns.push((current, key_str));
            current = SkipListNode::load_forward(current, 0);
        }

        for level in (0..self.max_level).rev() {
            write!(f, "Level {:2}: Head", level)?;
            let mut current_ptr = self.head;

            for (node, key_str) in &columns {
                let next_at_level = SkipListNode::load_forward(current_ptr, level);

                if !next_at_level.is_null() && next_at_level == *node {
                    write!(f, " -> [{}]", key_str)?;
                    current_ptr = next_at_level;
                } else {
                    let dashes = "-".repeat(key_str.len());
                    write!(f, " -> [{}]", dashes)?;
                }
            }

            writeln!(f, " -> Nil")?;
        }

        write!(f, "-----------------------------")
    }
}
