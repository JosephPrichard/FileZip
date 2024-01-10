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