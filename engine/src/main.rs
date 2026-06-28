use crate::skiplist::{SkipList, SkipListNode};

mod skiplist;
fn main() {
    let skip_list: SkipList<i32, i32> = SkipList::new(5, -1, -1);
    let skip_list_node: SkipListNode<i32, i32> = SkipListNode::new(5, &5, 5);
    println!("{:?}", skip_list);
    println!("{:?}", skip_list.random_level());
    println!("{:?}", skip_list.max_level);
    println!("{:?}", skip_list_node);
    let head = unsafe { &skip_list.head.unwrap().as_ref() };
    println!("{:?}", head);
    println!("{:?}", head.forward);
    println!("{:?}", head.key);
    println!("{:?}", head.value);
    println!("{:?}", head.level);
}
