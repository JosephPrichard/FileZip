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
    let args: Vec<String> = env::args().collect();

    let mut entries: Vec<String> = vec![];
    let mut exec_flag: String = String::from("");

    for i in 1..args.len() {
        let arg= &args[i];
        if arg.chars().nth(0).unwrap() == '-' {
            exec_flag = String::from(arg);
        } else {
            entries.push(String::from(arg));
        }
    }

    if entries.len() < 1 {
        println!("Needs at least one file path as an argument");
    }
    let last = entries.len() - 1;

    match exec_flag.as_str() {
        "-l" => {
            let blocks = &decompress::get_file_blocks(&entries[last]);
            block::list_file_blocks(blocks);
        },
        "-d" => decompress::unarchive_zip(&entries[last]),
        "-c" => compress::archive_dir(&entries),
        _ => compress::archive_dir(&entries)
    }
}
