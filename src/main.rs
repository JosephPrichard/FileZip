// Joseph Prichard
// 1/5/2023
// Application to compress or decompress files

use std::{env, io};
use crate::read::FileReader;

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
        // invariant: a program argument must have at least 1 character
        let first = arg.chars().nth(0).expect("Expected an argument to be at least 1 char");
        // check the arg type with
        if first == '-' {
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

    let flags = ExecFlags {
        exec_flag: &exec_flag,
        has_mt_flag,
    };
    match exec_cli(&flags, &entries) {
        Ok(()) => println!("Finished execution with success code"),
        Err(e) => panic!("IO error occurred during execution: {}", e.to_string())
    }
}

struct ExecFlags<'a> {
    exec_flag: &'a str,
    has_mt_flag: bool
}

fn exec_cli<'a>(exec_flags: &'a ExecFlags, entries: &Vec<String>) -> io::Result<()> {
    let last = entries.len() - 1;
    // execute a different command based on flag
    match exec_flags.exec_flag {
        "-l" | "list" => {
            let arg = &entries[last];
            let blocks_reader = &mut FileReader::new(arg)?;
            let blocks = &decompress::get_file_blocks(blocks_reader)?;
            data::list_file_blocks(blocks);
            Ok(())
        }
        "-d" | "decompress" => decompress::unarchive_zip(&entries[last], exec_flags.has_mt_flag),
        "-c" | "compress" | _ => compress::archive_dir(&entries, exec_flags.has_mt_flag),
    }
}