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

    let mut target: String = String::from("");
    let mut flag: String = String::from("");

    for arg in args {
        if arg.chars().nth(0).unwrap() == '-' {
            flag = String::from(arg);
        } else {
            target = String::from(arg);
        }
    }

    match flag.as_str() {
        "-l" => {
            let blocks = &decompress::get_file_blocks(&target);
            block::list_file_blocks(blocks);
        },
        "-d" => decompress::unarchive_zip(&target),
        "-c" => compress::archive_dir(&target),
        _ => compress::archive_dir(&target)
    }
}
