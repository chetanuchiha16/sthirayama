use std::{fmt::Debug, marker::PhantomData, ptr::NonNull};

use bitcode::{DecodeOwned, Encode};

use crate::skiplist::{SkipList, SkipListKV, SkipListNode};

pub trait TypeSkipListKey: Encode + DecodeOwned + Clone + PartialOrd + Debug {}

impl<K: Encode + DecodeOwned + Clone + PartialOrd + Debug> TypeSkipListKey for K {}
pub trait TypeSkipListValue: Encode + DecodeOwned + Debug + Clone {}

impl<V: Encode + DecodeOwned + Debug + Clone> TypeSkipListValue for V {}

pub struct SkipListIterator<'a, K, V> {
    // skiplist : SkipList<K, V>,
    pub next_node: Option<NonNull<SkipListNode<K, V>>>,
    pub marker: PhantomData<&'a SkipListNode<K, V>>, // pub kv : SkipListKV<K, V>
}

impl<'a, K: TypeSkipListKey, V: TypeSkipListValue> Iterator for SkipListIterator<'a, K, V> {
    type Item = &'a SkipListKV<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        // let head = &self.skiplist.head.unwrap();
        // let mut current = SkipListNode::get_forward(head)[0];
        // if let Some(cur_node) = current {
        //     let cur_kv = SkipListNode::get_data(&cur_node);
        //     let next_node = SkipListNode::get_forward(&cur_node)[0];
        //     current = next_node;
        //     Some(cur_kv.clone())

        // } else {
        //     None
        // }

        self.next_node.map(|cur_node| {
            self.next_node = SkipListNode::get_forward(&cur_node)[0];
            SkipListNode::get_data(&cur_node)
        })

        // if let Some(cur_node) = self.next_node {
        //     self.next_node = SkipListNode::get_forward(&cur_node)[0];
        //     // let c = SkipListNode::get_data(cur_node);
        //     let c = unsafe { cur_node.as_ref() };
        //     Some(c)
        // } else {
        //     None
        // }

        // todo!()
    }
}
