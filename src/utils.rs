// Joseph Prichard
// 1/5/2023
// Utilities for files, sizes, and conversions

use std::{env, fs};
use std::path::Path;
use std::fs::File;
use rand::{distributions::Alphanumeric, Rng};

pub fn get_size_of<T>(_: T) -> usize {
    std::mem::size_of::<T>()
}

// converts a string to a u64 in bytes
pub const fn str_to_u64(str: &str) -> u64 {
    let mut buffer = [0u8; 8];
    let mut i = 0;
    // converts a string to a buffer
    while i < str.len() && i < 8 {
        buffer[i] = str.as_bytes()[i];
        i += 1;
    }
    // converts a buffer to a u64
    u64::from_le_bytes(buffer)
}

// sums the sizes of all entries under a directory
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

pub fn get_no_ext(path: &str) -> String {
    Path::new(path)
        .with_extension("")
        .display()
        .to_string()
}

pub fn random_name() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

pub fn random_dir() -> String {
    String::from(env::current_dir()
        .expect("Can't get current dir")
        .to_str()
        .expect("Can't extract dir from option"))
        + "\\"
        + &random_name()
}

pub fn create_temp_file(dir: &str) -> (String, File) {
    let name = format!("{}\\{}", dir, random_name());
    println!("Creating tmp file: {}", &name);
    let file = File::create(&name).expect("Failed to create temp file");
    (name, file)
}

pub fn create_temp_files(dir: &str, count: u32) -> Vec<(String, File)> {
    let mut files = vec![];
    // generate a number of files with a random file name
    for _ in 0..count {
        let temp_file = create_temp_file(dir);
        files.push(temp_file);
    }
    files
}

pub fn teardown_temp_files(files: &[(String, File)]) {
    for file in files {
        println!("Deleting tmp file: {}", &file.0);
        fs::remove_file(&file.0).expect("Failed to delete temp file");
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use rand::Rng;
    use super::*;

    #[test]
    fn test_str_to_u64() {
        assert_eq!(str_to_u64("hello"), 478560413032);
        assert_eq!(str_to_u64("world"), 431316168567);
    }

    #[test]
    fn test_get_no_ext() {
        assert_eq!(get_no_ext("foo.txt"), "foo");
    }

    #[test]
    fn test_dir_entry_size() {
        let mut total_size = 0;
        let dir = random_dir();

        fs::create_dir(&dir).expect("Cannot create temp dir");

        // create temp files each with a randomly sized buffer
        let mut temp_files = create_temp_files(&dir, 10);
        for file in temp_files.iter_mut() {
            let file_size = rand::thread_rng().gen_range(1..1000) as usize;
            total_size += file_size as u64;

            let buffer = vec![1u8; file_size];
            file.1.write(&buffer).expect("Failed to write test data");
        }

        assert_eq!(dir_entry_size(Path::new(&dir)), total_size);

        teardown_temp_files(&temp_files);
        fs::remove_dir(&dir).expect("Cannot delete temp dir");
    }
}