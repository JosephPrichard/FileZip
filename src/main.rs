use std::fs;
use std::path::Path;

mod compress;
mod read;
mod decompress;
mod bitwise;
mod write;
mod tree;
mod debug;
mod block;

fn main() {
    let a = "C:\\Users\\Joseph\\OneDrive\\Documents\\Programs\\Rust\\Zipper\\input";
    compress::archive_entries(&vec![a]).expect("Failed to archive entries");
    println!("\nFinished");
}
