use std::fs;
use std::path::Path;

pub fn get_size_of<T>(_: T) -> usize {
    std::mem::size_of::<T>()
}

pub const fn str_to_u64(str: &str) -> u64 {
    let mut buffer = [0u8; 8];
    let mut i = 0;
    while i < str.len() && i < 8 {
        buffer[i] = str.as_bytes()[i];
        i += 1;
    }
    u64::from_le_bytes(buffer)
}

pub fn dir_entry_size(path: &Path) -> u64 {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Can't read directory") {
            let entry = entry.expect("Entry is invalid");
            let path = entry.path();
            size += dir_entry_size(&path);
        }
    } else {
        size += path.metadata().expect("Can't get metadata").len();
    }
    size
}

pub fn get_dir_name(dir: &str) -> &str {
    Path::new(dir)
        .file_name()
        .expect("Couldn't get directory name")
        .to_str()
        .expect("Couldn't convert to string")
}

pub fn get_parent_name(dir: &str) -> &str {
     Path::new(dir)
        .parent()
        .expect("Couldn't get parent")
        .to_str()
        .expect("Couldn't convert to string")
}