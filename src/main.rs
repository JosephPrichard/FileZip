// Joseph Prichard
// 1/5/2023
// Application to compress or decompress files

use std::env;
use std::thread::available_parallelism;

mod compress;
mod read;
mod decompress;
mod bitwise;
mod write;
mod tree;
mod debug;
mod data;
mod charset;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut entries: Vec<String> = vec![];
    let mut exec_flag: String = String::from("");
    let mut has_mt_flag: bool = false;

    // parse arguments to program
    for i in 1..args.len() {
        let arg= &args[i];
        // check the arg type
        if arg.chars().nth(0).unwrap() == '-' {
            // if the arg begins with a -, then the arg is a flag
            let flag = String::from(arg);
            if flag == "-mt" {
                has_mt_flag = true;
            } else {
                exec_flag = flag;
            }
        } else {
            // otherwise the arg is the file
            entries.push(String::from(arg));
        }
    }

    if entries.len() < 1 {
        println!("Needs at least one file path as an argument");
    }
    let last = entries.len() - 1;

    // configure the rayon thread pool based on -mt flag
    let threads = if has_mt_flag { available_parallelism().unwrap().get() } else { 1 };
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    // execute a different command based on flag
    match exec_flag.as_str() {
        "-l" => {
            let blocks = &decompress::get_file_blocks(&entries[last]);
            data::list_file_blocks(blocks);
        },
        "-d" => decompress::unarchive_zip(&entries[last]),
        "-c" => compress::archive_dir(&entries),
        _ => compress::archive_dir(&entries)
    }
}
