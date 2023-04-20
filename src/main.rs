// Joseph Prichard
// 1/5/2023
// Application to compress or decompress files

use std::env;

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
mod parallelism;

fn main() {
    let args: Vec<String> = env::args().collect();

    // arguments for execution
    let mut entries: Vec<String> = vec![];
    let mut exec_flag: String = String::from("");
    let mut has_mt_flag: bool = false;

    // parse arguments to program
    for i in 1..args.len() {
        let arg = &args[i];
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
        return;
    }
    let last = entries.len() - 1;

    // execute a different command based on flag
    match exec_flag.as_str() {
        "-l" => {
            let blocks = &decompress::get_file_blocks(&entries[last]);
            data::list_file_blocks(blocks);
        }
        "-d" => decompress::unarchive_zip(&entries[last], has_mt_flag),
        "-c" => compress::archive_dir(&entries, has_mt_flag),
        _ => compress::archive_dir(&entries, has_mt_flag)
    }
}
