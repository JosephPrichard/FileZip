// Joseph Prichard
// 1/5/2023
// Type definitions and utilities for blocks in an archive

use crate::utils::get_size_of;

// represents the metadata of a file within a compressed archive
#[derive(Clone)]
pub struct FileBlock {
    // relative name of file to base directory in archive
    pub filename_rel: String,
    // byte offset position of compressed data in archive
    pub file_byte_offset: u64,
    // original file size
    pub og_byte_size: u64,
    // length of encoded tree structure in bits
    pub tree_bit_size: u64,
    // length of compressed data in bits
    pub data_bit_size: u64,
}

impl FileBlock {
    pub fn get_header_size(&self) -> u64 {
        // string len calculation includes null terminator
        let s = self.filename_rel.as_bytes().len() + 1
            + get_size_of(self.tree_bit_size)
            + get_size_of(self.data_bit_size)
            + get_size_of(self.file_byte_offset)
            + get_size_of(self.og_byte_size);
        s as u64
    }
}

macro_rules! fb_row_format {
    () => ("{:>15}\t\t{:>15}\t\t{:>8}\t\t{:25}")
}

pub fn list_file_blocks(blocks: &[FileBlock]) {
    println!(fb_row_format!(), "compressed", "uncompressed", "ratio", "uncompressed_name");
    for block in blocks {
        let total_byte_size = (block.data_bit_size + block.tree_bit_size) / 8;
        let ratio_str = format!("{:.2}%", (total_byte_size as f64) / (block.og_byte_size as f64) * 100.0);
        println!(fb_row_format!(), total_byte_size, block.og_byte_size, &ratio_str, &block.filename_rel);
    }
    println!();
}

