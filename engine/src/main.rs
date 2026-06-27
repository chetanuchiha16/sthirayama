use crate::skiplist::{SkipList, SkipListNode};

mod skiplist;
fn main() {
    let skip_list: SkipList = SkipList::new(5);
    let skip_list_node: SkipListNode<i32, i32> = SkipListNode::new(5, &5, 5);
    println!("{:?}", skip_list);
    println!("{:?}", skip_list_node);
}
