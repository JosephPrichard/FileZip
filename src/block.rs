use std::error::Error;
use std::{mem, slice};
use std::intrinsics::transmute;
use crate::bitwise::SymbolCode;
use crate::tree::Tree;

// sizes of fields for header in bytes
pub const NAME_SIZE: usize = 256;
pub const BLOCK_SIZE: usize = NAME_SIZE + 4 + 4;

pub struct CodeBook {
    pub symbol_table: Vec<SymbolCode>,
    pub tree: Tree
}

// a part of a compressed archive
pub struct FileBlock {
    // full name of file including path
    pub filename: String,
    // length of compressed data in bits
    pub file_bit_size: u32,
    // byte offset position of compressed data in archive
    pub file_offset: u32,
    // code book for compressing the file to the archive
    // a code book is optional because it isn't present in the block until created
    pub code_book: Option<CodeBook>
}

impl FileBlock {
    pub fn new(filename: &str) -> FileBlock {
        FileBlock {
            filename: String::from(filename),
            file_bit_size: 0,
            file_offset: 0,
            code_book: None
        }
    }
}