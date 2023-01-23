// Joseph Prichard
// 1/5/2023
// Bit-by-bit file decompressor

use std::error::Error;
use crate::bitwise::SymbolCode;
use crate::debug::debug_tree;
use crate::read::FileReader;
use crate::tree::Node;
use crate::write::FileWriter;

pub fn decompress_file(input_filepath: &str, output_filepath: &str) {
    let reader = &mut FileReader::new(input_filepath).expect("Failed to create file reader");
    let root = read_node(reader).expect("Failed to decode tree from file");
    do_decompression(reader, output_filepath, &root).expect("Failed to decompress file");
}

pub fn read_node(reader: &mut FileReader) -> Result<Box<Node>, Box<dyn Error>> {
    let bit = reader.read_bit()?;
    if bit == 1 {
        Ok(Box::new(Node::leaf(reader.read_bits(8)?, 0)))
    } else {
        let left = read_node(reader)?;
        let right = read_node(reader)?;
        Ok(Box::new(Node::internal(left, right, 0, 0)))
    }
}

pub fn decompress_next_symbol(reader: &mut FileReader, writer: &mut FileWriter, node: &Box<Node>) -> Result<(), Box<dyn Error>> {
    if node.is_leaf() {
        writer.write_byte(node.plain_symbol)?;
    } else {
        let bit = reader.read_bit()?;
        if bit == 0 {
            let left = node.left.as_ref().expect("Expected left node to be Some");
            decompress_next_symbol(reader, writer, left)?;
        } else {
            let right = node.right.as_ref().expect("Expected right node to be Some");
            decompress_next_symbol(reader, writer, right)?;
        }
    }
    Ok(())
}

pub fn do_decompression(reader: &mut FileReader, output_filepath: &str, root: &Box<Node>) -> Result<(), Box<dyn Error>> {
    let writer = &mut FileWriter::new(output_filepath)?;

    debug_tree(root, SymbolCode::new());
    while !reader.eof() {
       decompress_next_symbol(reader, writer, root)?;
    }

    Ok(())
}