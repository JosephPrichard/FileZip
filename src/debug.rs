// Joseph Prichard
// 1/5/2023
// Utilities used for debugging only

use crate::bitwise::SymbolCode;
use crate::read::{BitReader, FileReader};
use crate::tree::Tree;

pub fn debug_binary_file(filepath: &str) {
    let mut reader = FileReader::new(filepath)
        .expect("Cannot create reader in debugger");
    println!();
    let mut c = 0;
    while !reader.eof() {
        let bit = reader.read_bit()
            .expect("Cannot read bit in debugger");
        print!("{}", bit);
        if (c + 1) % 4 == 0 {
            print!(" ");
        }
        c += 1;
    }
}

pub fn debug_tree_file(filepath: &str) {
    let mut reader = FileReader::new(filepath)
        .expect("Cannot create reader in debugger");
    println!();
    while !reader.eof() {
        let bit = reader.read_bit()
            .expect("Cannot read bit in debugger");
        print!("{}", bit);
        if bit > 0 {
            let byte = reader.read_bits(8)
                .expect("Cannot read bits in debugger");
            print!("{}", byte as char);
        }
    }
}

pub fn debug_tree(node: &Box<Tree>, symbol_code: SymbolCode) {
    if node.is_leaf() {
        println!("Leaf: {:#b} {} {}", symbol_code.encoded_symbol, symbol_code.bit_len, node.plain_symbol as char);
    }
    if let Some(left) = &node.left {
        let symbol_code = symbol_code.append_bit(0);
        debug_tree(left, symbol_code);
    }
    if let Some(right) = &node.right {
        let symbol_code = symbol_code.append_bit(1);
        debug_tree(right, symbol_code);
    }
}


