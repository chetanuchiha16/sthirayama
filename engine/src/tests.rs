use std::{
    io::{Write, stdin, stdout},
    time::Instant,
};

use crate::{
    engine_error,
    memtable::Memtable,
    skiplist::{SkipList, SkipListKV, SkipListNode},
    skiplist_error,
    sstable::{reader::SstableReader, writer::SstableWriter},
    wal::Wal,
};

pub fn try_new_skiplist() -> Result<(), skiplist_error::SkipListError> {
    println!("creating new skiplist...");
    let mut skip_list: SkipList<i32, i32> = SkipList::new(5, i32::MIN, -1);
    skip_list.insert(10, 100)?;
    skip_list.insert(20, 200)?;
    skip_list.insert(5, 50)?;
    skip_list.insert(15, 150)?;
    println!("{}", skip_list);

    println!("{:?}", skip_list.search(5)); // Some(50)
    println!("{:?}", skip_list.search(10)); // Some(100)
    println!("{:?}", skip_list.search(15)); // Some(150)
    println!("{:?}", skip_list.search(20)); // Some(200)
    println!("{:?}", skip_list.search(99)); // None
    Ok(())
}

pub fn try_wal() -> Result<(), skiplist_error::SkipListError> {
    let mut skip_list: SkipList<i32, i32> = SkipList::new(5, i32::MIN, -1);
    let start = Instant::now();

    // Call the function you are testing
    println!("----before recovering the skiplist----");
    println!("{}", skip_list);
    println!("recovering......");
    let mut wal = Wal::new()?;
    // wal.append(5, 6).unwrap();
    wal.recover::<i32, i32>(&mut skip_list)?;
    println!("----after recovering the skiplist----");
    println!("{}", skip_list);

    // Calculate elapsed time
    let duration = start.elapsed();

    println!("recovering took: {:?}", duration);
    Ok(())
}

pub fn pring_skiplist_details() -> Result<(), skiplist_error::SkipListError> {
    let mut skip_list: SkipList<i32, i32> = SkipList::new(5, -1, -1);
    skip_list.insert(6, 6)?;
    let skip_list_node = unsafe { SkipListNode::new(5, 5, 5).as_ref() };
    println!("{:?}", skip_list);
    println!("{:?}", skip_list.random_level());
    println!("{:?}", skip_list.max_level);
    println!("{:?}", skip_list_node);
    let head = unsafe { skip_list.head.as_mut() };
    println!("{:?}", head);
    println!("{:?}", head.forward);
    println!("{:?}", head.data.key);
    println!("{:?}", head.data.value);
    println!("{:?}", head.level);
    // head.forward[0] = SkipListNode::new(2, &5, 6);
    println!("{:?}", skip_list.search(6));
    Ok(())
}

pub fn cli(mut skiplist: SkipList<Vec<u8>, Vec<u8>>) -> Result<(), engine_error::EngineError> {
    loop {
        print!("sthirayama> ");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input).expect("input error");
        // let mut command: Vec<&str> = input.trim().split(" ").collect();
        let command: Vec<&str> = input.trim().split(" ").collect();
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
                    println!("{}", String::from_utf8(val)?);
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

pub fn test_block_split() -> Result<(), engine_error::EngineError> {
    // let mut skip_list: SkipList<Vec<u8>, Vec<u8>> = SkipList::new(5, vec![b'0'], vec![b'0']);
    let mut memtable = Memtable::new();
    let mut size = 0usize;
    while size <= 8000 {
        let key = fastrand::usize(1..=1000).to_string().as_bytes().to_vec();
        let value = fastrand::usize(1..=4000).to_string().as_bytes().to_vec();
        let data = SkipListKV::new(key, value);
        memtable.insert(&data.key.clone(), data.value.clone())?;

        let data_bytes = bitcode::encode(&data);
        let data_len = data_bytes.len();
        let data_len_bytes_len = data_len.to_le_bytes().len();
        size += data_len + data_len_bytes_len;
    }
    // skip_list.insert("10".as_bytes().to_vec(), "1".as_bytes().to_vec())?;
    // skip_list.insert("20".as_bytes().to_vec(), "2".as_bytes().to_vec())?;
    // skip_list.insert("30".as_bytes().to_vec(), "3".as_bytes().to_vec())?;

    let key = 99.to_string().as_bytes().to_vec();
    let value = 6.to_string().as_bytes().to_vec();
    let data = SkipListKV::new(key, value);
    memtable.insert(&data.key.clone(), data.value.clone())?;

    let mut s = SstableWriter::new("sstable.sst", memtable)?;
    s.write()?;
    // s.read();
    Ok(())
}

pub fn test_sstable_read() -> Result<(), engine_error::EngineError> {
    let mut sstable_reader = SstableReader::new("sstable.sst")?;
    // sstable_reader.read_footer();
    // sstable_reader.read_index()?;
    let num = 99;
    let key = num.to_string().as_bytes().to_vec();
    // sstable_reader.binary_search_index(&key);
    // sstable_reader.read_data_block(&key)?;
    let value = sstable_reader.binary_search_data(&key)?.unwrap();
    let key_val = str::from_utf8(&key).unwrap();
    let value_val = str::from_utf8(&value).unwrap();

    println!("found value {} for the key {}", value_val, key_val);
    Ok(())
}
