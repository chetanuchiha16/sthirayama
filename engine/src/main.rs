use crate::skiplist::{SkipList, SkipListNode};

mod skiplist;
fn main() {
    let skip_list: SkipList<i32, i32> = SkipList::new(5, -1, -1);
    let skip_list_node = unsafe { SkipListNode::new(5, &5, 5).unwrap().as_ref() };
    println!("{:?}", skip_list);
    println!("{:?}", skip_list.random_level());
    println!("{:?}", skip_list.max_level);
    println!("{:?}", skip_list_node);
    let head = unsafe { skip_list.head.unwrap().as_mut() };
    println!("{:?}", head);
    println!("{:?}", head.forward);
    println!("{:?}", head.key);
    println!("{:?}", head.value);
    println!("{:?}", head.level);
    // head.forward[0] = SkipListNode::new(2, &5, 6);
    skip_list.insert(5, 6);
    println!("{:?}", skip_list.search(5));
}
