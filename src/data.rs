// Joseph Prichard
// 1/5/2023
// Type definitions and utilities for blocks in an archive

use std::fmt::Display;
use crate::bitwise::SymbolCode;
use crate::tree::CodeTree;
use crate::utils::get_size_of;

pub const TABLE_SIZE: usize = 256;

// represents the metadata of a file within a compressed archive
#[derive(Clone)]
pub struct FileBlock {
    // full name of file including path
    pub filename_abs: String,
    // relative name of file to base directory in archive
    pub filename_rel: String,
    // byte offset position of compressed data in archive
    pub file_byte_offset: u64,
    // original file size
    pub og_byte_size: u64,
    // sizes of metadata in bits
    pub fbs: FileBitSize,
}

#[derive(Clone)]
pub struct FileBitSize {
    // length of encoded tree structure in bits
    pub tree_bit_size: u64,
    // length of compressed data in bits
    pub data_bit_size: u64,
}

// represents a codebook containing the data necessary to compress a file
pub struct CodeBook {
    // the symbol table of the file
    pub symbol_table: Box<[SymbolCode; TABLE_SIZE]>,
    // the tree structure to compress symbols
    pub tree: CodeTree,
    // metadata of the file to be compressed
    pub block: FileBlock,
}

impl FileBlock {
    pub fn from_fs_metadata(filename_rel: &str, filename_abs: &str, og_byte_size: u64) -> FileBlock {
        FileBlock {
            filename_abs: String::from(filename_abs),
            filename_rel: String::from(filename_rel),
            fbs: FileBitSize {
                tree_bit_size: 0,
                data_bit_size: 0,
            },
            file_byte_offset: 0,
            og_byte_size,
        }
    }

    pub fn get_header_size(&self) -> u64 {
        // string len calculation includes null terminator
        (self.filename_rel.as_bytes().len() + 1 +
            get_size_of(self.fbs.tree_bit_size) +
            get_size_of(self.fbs.data_bit_size) +
            get_size_of(self.file_byte_offset) +
            get_size_of(self.og_byte_size)
        ) as u64
    }
}

macro_rules! fb_row_format {
    () => ("{:>15}\t\t{:>15}\t\t{:>8}\t\t{:25}")
}

pub fn list_file_blocks(blocks: &[FileBlock]) {
    println!(fb_row_format!(), "compressed", "uncompressed", "ratio", "uncompressed_name");
    for block in blocks {
        let total_byte_size = (block.fbs.data_bit_size + block.fbs.tree_bit_size) / 8;
        let ratio_str = format!("{:.2}%", (total_byte_size as f64) / (block.og_byte_size as f64) * 100.0);
        println!(fb_row_format!(), total_byte_size, block.og_byte_size, &ratio_str, &block.filename_rel);
    }
    println!();
}

