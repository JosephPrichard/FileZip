// Joseph Prichard
// 1/5/2023
// Byte-by-byte file compressor

use std::collections::{BinaryHeap};
use std::{fs, path};
use std::path::{Path};
use std::time::Instant;
use crate::bitwise::SymbolCode;
use crate::debug::{debug_binary_file, debug_tree_file, debug_tree};
use crate::block::{CodeBook, FileBlock};
use crate::charset::{GRP_SEP, REC_SEP, SIG};
use crate::tree::{Node, Tree};
use crate::read::FileReader;
use crate::{block, utils};
use crate::write::FileWriter;

const TABLE_SIZE: usize = 256;

pub fn archive_dir(input_dir: &str) {
    let now = Instant::now();

    let mut blocks = get_file_blocks(input_dir);
    create_code_books(&mut blocks);

    let archive_filename = &format!("{}{}", input_dir, ".zipr");
    let writer = &mut FileWriter::new(archive_filename);
    writer.write_u64(SIG);

    write_block_headers(writer, &mut blocks);
    compress_files(writer, &blocks);

    let elapsed = now.elapsed();
    println!("Finished zipping in {:.2?}", elapsed);
    block::list_file_blocks(&blocks);
}

fn get_file_blocks(dir: &str) -> Vec<FileBlock> {
    let mut blocks = vec![];
    let path = Path::new(dir);
    walk_path(path.parent().expect("Failed to get parent path"), path, &mut blocks);
    blocks
}

fn walk_path(base_path: &Path, path: &Path, blocks: &mut Vec<FileBlock>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Can't read directory") {
            let entry = entry.expect("Entry is invalid");
            let path = entry.path();
            if path.is_dir() {
                walk_path(&base_path, &path, blocks);
            } else {
                let filename_abs = &String::from(path.to_str().unwrap());
                let filename_rel = &String::from(path
                    .strip_prefix(base_path)
                    .expect("Couldn't strip prefix from path")
                    .to_str()
                    .unwrap());
                let mut block = FileBlock::new(filename_rel, filename_abs);
                block.original_byte_size = utils::dir_entry_size(&path);
                blocks.push(block);
            }
        }
    }
}

fn create_code_books(blocks: &mut Vec<FileBlock>) {
    for block in blocks {
        create_code_book(block);
    }
}

fn create_code_book(block: &mut FileBlock) {
    let freq_table = create_freq_table(&block.filename_abs);
    let tree = create_code_tree(&freq_table);
    let symbol_table = create_code_table(&tree);
    // calculate the bit size for the file block for compressed data and for tree
    let mut char_count = 0;
    for i in 0..TABLE_SIZE {
        let freq = freq_table[i as usize];
        block.data_bit_size += freq * (symbol_table[i as usize].bit_len as u64);
        if freq > 0 {
            char_count += 1;
        }
    }
    block.tree_bit_size += 10 * char_count - 1;
    // add the code book to file block
    block.code_book = Some(CodeBook { symbol_table, tree });
}

fn write_block_headers(writer: &mut FileWriter, blocks: &mut Vec<FileBlock>) {
    // calculate the total block size for the header, including the grp sep byte
    let mut header_size = 1;
    for block in &*blocks {
        // header size plus an additional rec sep byte
        header_size += block.get_header_size() + 1;
    }
    // iterate through each block, calculate the file offset and write the block
    let mut total_offset = 0;
    for block in &mut *blocks {
        // write record sep to identify start of record
        writer.write_byte(REC_SEP);
        // calculate the file sizes and offsets for the block
        block.file_byte_offset = header_size + total_offset;
        total_offset += 1 + (block.data_bit_size + block.tree_bit_size) / 8;
        // write the block into memory
        writer.write_block(block);
    }
    // write group sep after headers are complete
    writer.write_byte(GRP_SEP);
}

fn compress_files(writer: &mut FileWriter, blocks: &Vec<FileBlock>) {
    for block in blocks {
        let code_book = block.code_book.as_ref().unwrap();
        write_node(writer, &code_book.tree.root);
        compress_file(&block.filename_abs, writer, &code_book.symbol_table);
        writer.align_to_byte();
    }
}

fn write_node(writer: &mut FileWriter, node: &Box<Node>) {
    if node.is_leaf() {
        writer.write_bit(1);
        writer.write_bits(node.plain_symbol, 8);
    } else {
        writer.write_bit(0);
        let left = node.left.as_ref().expect("Expected left node to be Some");
        write_node(writer, left);
        let right = node.right.as_ref().expect("Expected right node to be Some");
        write_node(writer, right);
    }
}

fn compress_file(input_filepath: &str, writer: &mut FileWriter, symbol_table: &Vec<SymbolCode>) {
    let mut reader = FileReader::new(input_filepath);
    while !reader.eof() {
        let byte = reader.read_byte();
        writer.write_symbol(&symbol_table[byte as usize]);
    }
}

fn create_freq_table(input_filepath: &str) -> Vec<u64> {
    let mut freq_table = vec![0u64; TABLE_SIZE];

    // iterate through each byte in the file and increment count
    let mut reader = FileReader::new(input_filepath);
    while !reader.eof() {
        let byte = reader.read_byte();
        freq_table[usize::from(byte)] += 1;
    }

    freq_table
}

fn create_code_tree(freq_table: &Vec<u64>) -> Tree {
    let mut heap = BinaryHeap::new();

    // add the frequency table nodes to priority queue
    let mut symbol_count = 0;
    for i in 0..TABLE_SIZE {
        let freq = freq_table[i];
        if freq != 0 {
            heap.push(Box::new(Node::leaf(i as u8, freq)));
            symbol_count += 1;
        }
    }

    // huffman coding algorithm
    while heap.len() > 1 {
        let first_node = heap.pop().expect("First node is None");
        let second_node = heap.pop().expect("Second node is None");
        let w = first_node.weight + second_node.weight;
        heap.push(Box::new(Node::internal(first_node, second_node, 0, w)));
    }

    let root = heap.pop().expect("Heap is empty after algorithm");
    Tree { root, symbol_count }
}

fn walk_code_tree(node: &Box<Node>, mut symbol_code: SymbolCode, symbol_table: &mut Vec<SymbolCode>) {
    if node.is_leaf() {
        symbol_code.plain_symbol = node.plain_symbol;
        symbol_table[usize::from(node.plain_symbol)] = symbol_code;
    }
    if let Some(left) = &node.left {
        let symbol_code = symbol_code.append_bit(0);
        walk_code_tree(left, symbol_code, symbol_table);
    }
    if let Some(right) = &node.right {
        let symbol_code = symbol_code.append_bit(1);
        walk_code_tree(right, symbol_code, symbol_table);
    }
}

fn create_code_table(tree: &Tree) -> Vec<SymbolCode> {
    let symbol_code = SymbolCode::new();
    let mut symbol_table = vec![symbol_code; TABLE_SIZE];
    walk_code_tree(&tree.root, symbol_code, &mut symbol_table);
    symbol_table
}