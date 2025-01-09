// Joseph Prichard
// 1/5/2023
// Type definitions and utilities for the binary tree structure used for huffman coding

use std::cmp::Ordering;

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

pub fn sizeof<T>(_: T) -> usize {
    std::mem::size_of::<T>()
}

impl FileBlock {
    pub fn get_header_size(&self) -> u64 {
        // string len calculation includes null terminator
        let size = 1 +
            self.filename_rel.as_bytes().len() +
            sizeof(self.tree_bit_size) +
            sizeof(self.data_bit_size) +
            sizeof(self.file_byte_offset) +
            sizeof(self.og_byte_size);
        size as u64
    }
}

#[derive(Clone, Copy)]
pub struct SymbolCode {
    pub plain_symbol: u8,
    pub encoded_symbol: u32,
    pub bit_len: u8,
}

impl SymbolCode {
    pub fn new() -> SymbolCode {
        SymbolCode { plain_symbol: 0, encoded_symbol: 0, bit_len: 0 }
    }

    pub fn append_bit(&self, bit: u32) -> SymbolCode {
        SymbolCode {
            plain_symbol: self.plain_symbol,
            encoded_symbol: self.encoded_symbol ^ (bit << self.bit_len),
            bit_len: self.bit_len + 1,
        }
    }
}

pub struct Tree {
    pub left: Option<Box<Tree>>,
    pub right: Option<Box<Tree>>,
    pub plain_symbol: u8,
    pub weight: u64,
}

impl Tree {
    // creates a leaf structure with no children
    pub fn leaf(symbol: u8, weight: u64) -> Tree {
        Tree {
            left: None,
            right: None,
            plain_symbol: symbol,
            weight,
        }
    }

    // moves the left and right nodes
    pub fn internal(left: Box<Tree>, right: Box<Tree>, symbol: u8, weight: u64) -> Tree {
        Tree {
            left: Some(Box::new(*left)),
            right: Some(Box::new(*right)),
            plain_symbol: symbol,
            weight,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left == None && self.right == None
    }
}

impl Eq for Tree {}

impl PartialEq<Self> for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl PartialOrd<Self> for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Self) -> Ordering {
        other.weight.cmp(&self.weight)
    }
}