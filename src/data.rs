// Joseph Prichard
// 1/5/2023
// Type definitions and utilities for blocks in an archive

use crate::bitwise::SymbolCode;
use crate::tree::Tree;
use crate::utils::get_size_of;

// represents the metadata of a file within a compressed archive
#[derive(Clone)]
pub struct FileBlock {
    // full name of file including path
    pub filename_abs: String,
    // relative name of file to base directory in archive
    pub filename_rel: String,
    // length of encoded tree structure in bits
    pub tree_bit_size: u64,
    // length of compressed data in bits
    pub data_bit_size: u64,
    // byte offset position of compressed data in archive
    pub file_byte_offset: u64,
    // original file size
    pub original_byte_size: u64,
}

// represents a codebook containing the data necessary to compress a file
pub struct CodeBook {
    // the symbol table of the file
    pub symbol_table: Vec<SymbolCode>,
    // the tree structure to compress symbols
    pub tree: Tree,
    // metadata of the file to be compressed
    pub block: FileBlock,
}

impl FileBlock {
    pub fn new(filename_rel: &str, filename_abs: &str) -> FileBlock {
        FileBlock {
            filename_abs: String::from(filename_abs),
            filename_rel: String::from(filename_rel),
            tree_bit_size: 0,
            data_bit_size: 0,
            file_byte_offset: 0,
            original_byte_size: 0,
        }
    }

    pub fn get_header_size(&self) -> u64 {
        // string len calculation includes null terminator
        (self.filename_rel.as_bytes().len() + 1 +
            get_size_of(self.tree_bit_size) +
            get_size_of(self.data_bit_size) +
            get_size_of(self.file_byte_offset) +
            get_size_of(self.original_byte_size)
        ) as u64
    }
}

pub fn list_file_blocks(blocks: &Vec<FileBlock>) {
    println!(
        "{:>15}\t\t{:>15}\t\t{:>8}\t\t{:25}",
        "compressed",
        "uncompressed",
        "ratio",
        "uncompressed_name"
    );
    for block in blocks {
        let total_byte_size = (block.data_bit_size + block.tree_bit_size) / 8;
        println!(
            "{:>15}\t\t{:>15}\t\t{:>8}\t\t{:25}",
            total_byte_size,
            block.original_byte_size,
            format!("{:.2}%", (total_byte_size as f64) / (block.original_byte_size as f64) * 100.0),
            block.filename_rel
        );
    }
    println!();
}

