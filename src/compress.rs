// Joseph Prichard
// 1/5/2023
// Byte-by-byte file compressor

use std::collections::{BinaryHeap};
use std::fs;
use std::path::{Path};
use std::time::Instant;
use rayon::prelude::*;
use crate::bitwise::SymbolCode;
use crate::data::{CodeBook, FileBlock};
use crate::{charset, parallelism};
use crate::tree::{Node, Tree};
use crate::read::FileReader;
use crate::{data, utils};
use crate::write::FileWriter;

const TABLE_SIZE: usize = 256;

pub fn archive_dir(input_entry: &[String], multithreaded: bool) {
    let now = Instant::now();

    // get the blocks for each file we need to compress
    let mut blocks = get_file_blocks(input_entry);

    // configure parallelism
    parallelism::configure_thread_pool(multithreaded, blocks.len());

    // generate code books (parallelized)
    let mut code_books = create_code_books(&mut blocks);

    // create the output archive file and its writer
    let archive_filename = &format!("{}{}", input_entry[0], ".zipr");
    let writer = &mut FileWriter::new(archive_filename);
    writer.write_u64(charset::SIG);

    // write the headers to the file, and then compress each file into the output archive
    write_block_headers(writer, &mut code_books);
    compress_files(writer, &code_books);

    let elapsed = now.elapsed();
    println!("Finished zipping in {:.2?}", elapsed);
    data::list_file_blocks(&blocks);
}

fn get_file_blocks(entries: &[String]) -> Vec<FileBlock> {
    let mut blocks = vec![];
    for entry in entries {
        let path = Path::new(entry);
        walk_path(path.parent().expect("Failed to get parent path"), path, &mut blocks);
    }
    blocks
}

fn walk_path(base_path: &Path, path: &Path, blocks: &mut Vec<FileBlock>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Can't read directory") {
            let entry = entry.expect("Entry is invalid");
            let path = entry.path();
            walk_path(&base_path, &path, blocks);
        }
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

fn create_code_books(blocks: &mut [FileBlock]) -> Vec<CodeBook> {
    // create code books, this operation can be parallelized because it only reads
    blocks.into_par_iter()
        .map(|block| create_code_book(block))
        .collect()
}

fn create_code_book(block: &mut FileBlock) -> CodeBook {
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
    // create the prepared file block containing the code book
    CodeBook {
        symbol_table,
        tree,
        block: block.clone(),
    }
}

fn write_block_headers(writer: &mut FileWriter, code_books: &mut [CodeBook]) {
    // calculate the total block size for the header, including the grp sep byte
    let mut header_size = 1;
    for code_book in code_books.iter_mut() {
        // header size plus an additional rec sep byte
        header_size += code_book.block.get_header_size() + 1;
    }
    // iterate through each block, calculate the file offset and write the block
    let mut total_offset = 0;
    for code_book in code_books.iter_mut() {
        // write record sep to identify start of record
        writer.write_byte(charset::REC_SEP);
        // calculate the file sizes and offsets for the block
        code_book.block.file_byte_offset = header_size + total_offset;
        total_offset += 1 + (code_book.block.data_bit_size + code_book.block.tree_bit_size) / 8;
        // write the block into memory
        writer.write_block(&code_book.block);
    }
    // write group sep after headers are complete
    writer.write_byte(charset::GRP_SEP);
}

fn compress_files(writer: &mut FileWriter, code_books: &[CodeBook]) {
    for code_book in code_books {
        write_tree(writer, &code_book.tree.root);
        compress_file(&code_book.block.filename_abs, writer, &code_book.symbol_table);
        writer.align_to_byte();
    }
}

fn write_tree(writer: &mut FileWriter, node: &Box<Node>) {
    if node.is_leaf() {
        writer.write_bit(1);
        writer.write_bits(node.plain_symbol, 8);
    } else {
        writer.write_bit(0);
        let left = node.left.as_ref().expect("Expected left node to be Some");
        write_tree(writer, left);
        let right = node.right.as_ref().expect("Expected right node to be Some");
        write_tree(writer, right);
    }
}

fn compress_file(input_filepath: &str, writer: &mut FileWriter, symbol_table: &[SymbolCode]) {
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
        freq_table[byte as usize] += 1;
    }

    freq_table
}

fn create_code_tree(freq_table: &[u64]) -> Tree {
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

fn walk_code_tree(node: &Box<Node>, mut symbol_code: SymbolCode, symbol_table: &mut [SymbolCode]) {
    if node.is_leaf() {
        symbol_code.plain_symbol = node.plain_symbol;
        symbol_table[node.plain_symbol as usize] = symbol_code;
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