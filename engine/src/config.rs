use std::{fs::create_dir_all, path::PathBuf};

pub fn get_root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn get_sstable_path() -> PathBuf {
    let root = get_root_path();
    let sstable_path = root.join("sstable_files");
    create_dir_all(&sstable_path);
    sstable_path
}
