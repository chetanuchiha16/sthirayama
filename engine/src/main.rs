#![allow(unused)]
use std::{
    error::{self, Error},
    io::{Write, stdin, stdout},
    time::Instant,
};

use crate::{
    skiplist::{SkipList, SkipListNode},
    wal::Wal,
};
mod skiplist;
mod sstable;
#[cfg(test)]
mod test_skiplist;
mod wal;

fn try_new_skiplist() -> Result<(), Box<dyn Error>> {
    println!("creating new skiplist...");
    let mut skip_list: SkipList<i32, i32> = SkipList::new(5, i32::MIN, -1)?;
    skip_list.insert_with_wal(10, 100)?;
    skip_list.insert_with_wal(20, 200)?;
    skip_list.insert_with_wal(5, 50)?;
    skip_list.insert_with_wal(15, 150)?;
    println!("{}", skip_list);

    println!("{:?}", skip_list.search(5)); // Some(50)
    println!("{:?}", skip_list.search(10)); // Some(100)
    println!("{:?}", skip_list.search(15)); // Some(150)
    println!("{:?}", skip_list.search(20)); // Some(200)
    println!("{:?}", skip_list.search(99)); // None
    Ok(())
}

fn try_wal() -> Result<(), Box<dyn Error>> {
    let mut skip_list: SkipList<i32, i32> = SkipList::new(5, i32::MIN, -1)?;
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

fn pring_skiplist_details() -> Result<(), Box<dyn Error>> {
    let mut skip_list: SkipList<i32, i32> = SkipList::new(5, -1, -1)?;
    skip_list.insert(6, 6)?;
    let skip_list_node = unsafe { SkipListNode::new(5, 5, 5).as_ref() };
    println!("{:?}", skip_list);
    println!("{:?}", skip_list.random_level());
    println!("{:?}", skip_list.max_level);
    println!("{:?}", skip_list_node);
    let head = unsafe { skip_list.head.unwrap().as_mut() };
    println!("{:?}", head);
    println!("{:?}", head.forward);
    println!("{:?}", head.data.key);
    println!("{:?}", head.data.value);
    println!("{:?}", head.level);
    // head.forward[0] = SkipListNode::new(2, &5, 6);
    println!("{:?}", skip_list.search(6));
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // pring_skiplist_details()?;
    // try_new_skiplist()?;
    // try_wal()?;
    let mut skiplist: SkipList<Vec<u8>, Vec<u8>> = SkipList::new(5, vec![b'0'], vec![b'0'])?;

    loop {
        print!("sthirayama> ");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input).expect("input error");
        // let mut command: Vec<&str> = input.trim().split(" ").collect();
        let mut command: Vec<&str> = input.trim().split(" ").collect();
        if command.len() > 3 {
            println!("invalid");
        }
        // let command: (&str, &str, &str) = split.collect();
        // let (Some(command), Some(key), Some(value), None) =
        //     (split.next(), split.next(), split.next(), split.next())
        // else {
        //     println!("bad length");
        //     return Err("bad length".into());
        // };
        // println!("{:?}", command);
        match command[0] {
            "set" => {
                // let (key, value) = (command[0], command[1]);
                // skiplist.insert(command[1].parse()?, command[2].parse()?)?;
                skiplist.insert(
                    command[1].as_bytes().to_vec(),
                    command[2].as_bytes().to_vec(),
                )?;
            }
            "get" => {
                if let Some(val) = skiplist.search(command[1].as_bytes().to_vec()) {
                    println!("{:?}", String::from_utf8(val)?);
                } else {
                    println!("key does not exist");
                };
            }
            "\\q" => break,
            _ => {
                println!("invalid command");
                // break;
            }
        }
    }
    Ok(())
}
