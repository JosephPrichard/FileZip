// Joseph Prichard
// 1/5/2023
// Application to compress or decompress files

use std::{env, io};
use crate::compress::{archive_dir, list_file_blocks};
use crate::decompress::{get_file_blocks, unarchive_zip};
use crate::read::FileReader;

mod compress;
mod read;
mod decompress;
mod write;
mod structs;
mod threading;
mod bitwise;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut entries: Vec<String> = vec![];
    let mut exec_flag: String = String::from("");
    let mut has_mt_flag: bool = false;

    for i in 1..args.len() {
        let arg = &args[i];
        // invariant: a program argument must have at least 1 character
        let first = arg.chars().nth(0).expect("Expected an argument to be at least 1 char");

        if first == '-' {
            // if the arg begins with a -, then the arg is a flag
            let flag = String::from(arg);
            if flag == "-mt" {
                has_mt_flag = true;
            } else {
                exec_flag = flag;
            }
        } else {
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
    has_mt_flag: bool,
}

fn exec_cli<'a>(exec_flags: &'a ExecFlags, entries: &Vec<String>) -> io::Result<()> {
    let last = entries.len() - 1;
    // execute a different command based on flag
    match exec_flags.exec_flag {
        "-l" | "list" => {
            let archive_path = &entries[last];
            let blocks_reader = &mut FileReader::new(archive_path)?;
            let blocks = &get_file_blocks(blocks_reader)?;
            list_file_blocks(blocks);
            Ok(())
        }
        "-d" | "decompress" => {
            let archive_path = &entries[last];
            unarchive_zip(archive_path, exec_flags.has_mt_flag)
        }
        "-c" | "compress" | _ => {
            let blocks = archive_dir(&entries, exec_flags.has_mt_flag)?;
            list_file_blocks(&blocks);
            Ok(())
        }
    }
}

mod tests {
    use std::collections::HashMap;
    use std::fs;
    use crate::compress::archive_dir;
    use crate::decompress::unarchive_zip;

    #[test]
    fn test_compress_directory() {
        let input_path = String::from("./test/files");

        let mut dir_data = HashMap::new();
        for entry in fs::read_dir(&input_path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                continue
            }
            let file_data = fs::read_to_string(&path)
                .expect(&format!("Cannot read file at path {}", path.to_str().unwrap()));

            let relative_path = path.strip_prefix(&input_path).unwrap().to_owned();
            dir_data.insert(relative_path.clone(), file_data);
        }
        println!("Directory files {:?}", dir_data.keys());

        archive_dir(&[input_path], false).unwrap();
        unarchive_zip("./test/files.zipr", false).unwrap();

        let output_path = "./test/files/files";
        for entry in fs::read_dir(output_path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                continue
            }
            let file_data = fs::read_to_string(&path)
                .expect(&format!("Cannot read at file path {}", path.to_str().unwrap()));

            let relative_path = path.strip_prefix(&output_path).unwrap();
            let other_file_data = dir_data.get(relative_path)
                .expect(&format!("Cannot find path in map {}", path.to_str().unwrap()));

            if file_data != *other_file_data {
                panic!("File data for file path is different: {}", path.to_str().unwrap())
            }
        }

        fs::remove_dir_all("./test/files/files").unwrap();
    }
}