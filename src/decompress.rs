// Joseph Prichard
// 1/5/2023
// Bit-by-bit file decompressor

use std::fs;
use std::path;
use std::path::{Path};
use std::time::Instant;
use crate::bitwise::SymbolCode;
use crate::block::FileBlock;
use crate::charset::{GRP_SEP, SIG};
use crate::debug::debug_tree;
use crate::read::FileReader;
use crate::tree::Node;
use crate::utils::get_size_of;
use crate::write::FileWriter;

pub fn unarchive_zip(input_filepath: &str, output_dir: &str) {
    println!("Begin un-archival of directory");
    let now = Instant::now();

    println!("Iterate over block headers and decompress data segment");
    decompress_files(input_filepath, output_dir);

    let elapsed = now.elapsed();
    println!("Finished un-archival in {:.2?}", elapsed);
}

fn decompress_files(archive_filepath: &str, output_dir: &str) {
    let mut reader = FileReader::new(archive_filepath);
    if reader.read_u64() != SIG {
        panic!("File is not a zipr file");
    }
    // iterate through headers until the file separator byte is found or eof
    while !reader.eof() {
        let sep = reader.read_byte();
        if sep == GRP_SEP {
            break;
        }
        let block = reader.read_block();
        decompress_file(&block, output_dir, archive_filepath);
    }
}

fn decompress_file(block: &FileBlock, output_dir: &str, archive_filepath: &str) {
    let unarchived_filename = &format!("{}{}{}", output_dir, path::MAIN_SEPARATOR, &block.filename_rel);
    println!("Writing to {}", unarchived_filename);

    // read from the main archive jumping to the data segment
    let reader = &mut FileReader::new(archive_filepath);
    reader.seek_from_start((get_size_of(SIG) as u64) + block.file_byte_offset);

    println!("Reading node structure for block {}", block.filename_rel);
    let root = read_node(reader);

    println!("Writing decompressed symbols for block {}", block.filename_rel);
    let unarchived_parent = Path::new(unarchived_filename).parent().unwrap();
    fs::create_dir_all(unarchived_parent).expect("Couldn't create directories");

    // decompress each symbol in data segment, stopping at the end
    let writer = &mut FileWriter::new(unarchived_filename);
    let start_read_len = reader.read_len() as i64;
    while !reader.eof() {
        let read_len = reader.read_len() as i64;
        if (read_len - start_read_len) >= block.data_bit_size as i64 {
            break;
        }
        decompress_next_symbol(reader, writer, &root);
    }
}

fn read_node(reader: &mut FileReader) -> Box<Node> {
    let bit = reader.read_bit();
    if bit == 1 {
        Box::new(Node::leaf(reader.read_bits(8), 0))
    } else {
        let left = read_node(reader);
        let right = read_node(reader);
        Box::new(Node::internal(left, right, 0, 0))
    }
}

fn decompress_next_symbol(reader: &mut FileReader, writer: &mut FileWriter, node: &Box<Node>) {
    if node.is_leaf() {
        writer.write_byte(node.plain_symbol);
    } else {
        let bit = reader.read_bit();
        if bit == 0 {
            let left = node.left.as_ref().expect("Expected left node to be Some");
            decompress_next_symbol(reader, writer, left);
        } else {
            let right = node.right.as_ref().expect("Expected right node to be Some");
            decompress_next_symbol(reader, writer, right);
        }
    }
}