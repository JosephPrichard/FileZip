use std::env;

mod compress;
mod read;
mod decompress;
mod bitwise;
mod write;
mod tree;
mod debug;
mod block;
mod charset;
mod utils;

fn main() {
    let a = "C:\\Users\\Joseph\\OneDrive\\Documents\\Programs\\Rust\\Zipper\\input";
    let b = "C:\\Users\\Joseph\\OneDrive\\Documents\\Programs\\Rust\\Zipper\\output\\input.zipr";
    let c = "C:\\Users\\Joseph\\OneDrive\\Documents\\Programs\\Rust\\Zipper\\output";

    let args: Vec<String> = env::args().collect();

    compress::archive_dir(a, c);
    decompress::unarchive_zip(b, c);


}
