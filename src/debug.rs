use crate::bitwise::SymbolCode;
use crate::read::FileReader;
use crate::tree::Node;

pub fn debug_binary_file(filepath: &str) {
    let mut reader = FileReader::new(filepath);
    println!();
    let mut c = 0;
    while !reader.eof() {
        let bit = reader.read_bit();
        print!("{}", bit);
        if (c + 1) % 4 == 0 {
            print!(" ");
        }
        c += 1;
    }
}

pub fn debug_tree_file(filepath: &str) {
    let mut reader = FileReader::new(filepath);
    println!();
    while !reader.eof() {
        let bit = reader.read_bit();
        print!("{}", bit);
        if bit > 0 {
            let byte = reader.read_bits(8);
            print!("{}", byte as char);
        }
    }
}

pub fn debug_tree(node: &Box<Node>, symbol_code: SymbolCode) {
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


