use std::{error::Error, time::Instant};

use crate::{
    skiplist::{SkipList, SkipListNode},
    wal::Wal,
};
mod skiplist;
#[cfg(test)]
mod test_skiplist;
mod wal;
fn main() -> Result<(), Box<dyn Error>> {
    // let mut skip_list: SkipList<i32, i32> = SkipList::new(5, -1, -1)?;
    // skip_list.insert(6, 6)?;
    // let skip_list_node = unsafe { SkipListNode::new(5, 5, 5).as_ref() };
    // println!("{:?}", skip_list);
    // println!("{:?}", skip_list.random_level());
    // println!("{:?}", skip_list.max_level);
    // println!("{:?}", skip_list_node);
    // let head = unsafe { skip_list.head.unwrap().as_mut() };
    // println!("{:?}", head);
    // println!("{:?}", head.forward);
    // println!("{:?}", head.data.key);
    // println!("{:?}", head.data.value);
    // println!("{:?}", head.level);
    // // head.forward[0] = SkipListNode::new(2, &5, 6);
    // println!("{:?}", skip_list.search(6));

    // println!("creating new skiplist...");
    let mut skip_list: SkipList<i32, i32> = SkipList::new(5, i32::MIN, -1)?;
    // skip_list.insert_with_wal(10, 100)?;
    // skip_list.insert_with_wal(20, 200)?;
    // skip_list.insert_with_wal(5, 50)?;
    // skip_list.insert_with_wal(15, 150)?;
    // println!("{}", skip_list);

    // println!("{:?}", skip_list.search(5)); // Some(50)
    // println!("{:?}", skip_list.search(10)); // Some(100)
    // println!("{:?}", skip_list.search(15)); // Some(150)
    // println!("{:?}", skip_list.search(20)); // Some(200)
    // // println!("{:?}", skip_list);
    // println!("{:?}", skip_list.search(99)); // None
    // let mut skiplist = SkipList::new(5, -1, -1)?;

    let start = Instant::now();

    // Call the function you are testing
    println!("----before recovering the skiplist----");
    println!("{}", skip_list);
    println!("recovering......");
    let mut wal = Wal::new().unwrap();
    // wal.append(5, 6).unwrap();
    wal.recover::<i32, i32>(&mut skip_list)?;
    println!("----after recovering the skiplist----");
    println!("{}", skip_list);

    // Calculate elapsed time
    let duration = start.elapsed();

    println!("recovering took: {:?}", duration);
    Ok(())
}
