// Joseph Prichard
// 1/5/2023
// Byte-by-byte file compressor

use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use crate::bitwise::SymbolCode;
use crate::debug::{debug_binary_file, debug_tree_file, debug_tree};
use crate::block::{BLOCK_SIZE, CodeBook, FileBlock};
use crate::tree::{Node, Tree};
use crate::read::FileReader;
use crate::write::FileWriter;

const TABLE_SIZE: usize = 256;

pub fn archive_entries(entries: &Vec<&str>) -> Result<(), Box<dyn Error>> {
    println!("Fetching file blocks from entries");
    let mut blocks = get_file_blocks(entries)?;

    println!("Generating code books for compression");
    create_code_books(&mut blocks)?;

    let archive_filename = &format!("{}{}", entries[0], ".zip.txt");
    let writer = &mut FileWriter::new(archive_filename)?;

    println!("Writing blocks to archive");
    write_blocks(writer, &mut blocks)?;

    println!("Compressing file blocks to archive");
    compress_files(writer, &blocks)?;

    Ok(())
}

fn get_file_blocks(entries: &Vec<&str>) -> Result<Vec<FileBlock>, Box<dyn Error>> {
    // collect entries into a list of file blocks to be archived
    let mut blocks = vec![];
    for entry in entries {
        walk_path(Path::new(entry), &mut blocks)?;
    }
    Ok(blocks)
}

fn walk_path(path: &Path, files: &mut Vec<FileBlock>) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_path(&path, files)?;
            } else {
                let filename = &String::from(path.to_str().unwrap());
                files.push(FileBlock::new(filename));
            }
        }
    }
    Ok(())
}

fn create_code_books(blocks: &mut Vec<FileBlock>) -> Result<(), Box<dyn Error>> {
    for block in blocks {
        create_code_book(block)?;
    }
    Ok(())
}

fn create_code_book(block: &mut FileBlock) -> Result<(), Box<dyn Error>> {
    let freq_table = create_freq_table(&block.filename)?;
    let tree = create_code_tree(&freq_table)?;
    let symbol_table = create_code_table(&tree)?;

    // calculate the bit size for the file block
    for i in 0..TABLE_SIZE {
        block.file_bit_size += freq_table[i as usize] * (symbol_table[i as usize].bit_len as u32);
    }

    block.code_book = Some(CodeBook { symbol_table, tree });

    Ok(())
}

fn write_blocks(writer: &mut FileWriter, blocks: &mut Vec<FileBlock>) -> Result<(), Box<dyn Error>> {
    let blocks_size = (BLOCK_SIZE * blocks.len()) as u32;
    let mut total_offset = 0;
    for block in blocks {
        // calculate the file sizes and offsets for the block
        let byte_len = ((block.file_bit_size + 7) / 8) * 8;
        block.file_offset = blocks_size + total_offset + byte_len;
        total_offset += byte_len;
        // write the block into memory
        writer.write_block(block)?;
    }
    Ok(())
}

fn compress_files(writer: &mut FileWriter, blocks: &Vec<FileBlock>) -> Result<(), Box<dyn Error>> {
    for block in blocks {
        let code_book = block.code_book.as_ref().unwrap();
        write_node(writer, &code_book.tree.root)?;
        compress_file(&block.filename, writer, &code_book.symbol_table)?;
    }
    Ok(())
}

fn write_node(writer: &mut FileWriter, node: &Box<Node>) -> Result<(), Box<dyn Error>> {
    if node.is_leaf() {
        writer.write_bit(1)?;
        writer.write_bits(node.plain_symbol, 8)?;
    } else {
        writer.write_bit(0)?;
        let left = node.left.as_ref().expect("Expected left node to be Some");
        write_node(writer, left)?;
        let right = node.right.as_ref().expect("Expected right node to be Some");
        write_node(writer, right)?;
    }
    Ok(())
}

fn compress_file(input_filepath: &str, writer: &mut FileWriter, symbol_table: &Vec<SymbolCode>) -> Result<(), Box<dyn Error>> {
    let mut reader = FileReader::new(input_filepath)?;

    while !reader.eof() {
        let byte = reader.read_byte()?;
        writer.write_symbol(&symbol_table[byte as usize])?;
    }

    Ok(())
}

fn create_freq_table(input_filepath: &str) -> Result<Vec<u32>, Box<dyn Error>> {
    // keep track of symbol counts
    let mut freq_table = vec![0u32; TABLE_SIZE];

    // iterate through each byte in the file
    let mut reader = FileReader::new(input_filepath)?;
    while !reader.eof() {
        let byte = reader.read_byte()?;
        freq_table[usize::from(byte)] += 1;
    }

    Ok(freq_table)
}

fn create_code_tree(freq_table: &Vec<u32>) -> Result<Tree, Box<dyn Error>> {
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
        let first_node = heap.pop().ok_or("First node is None")?;
        let second_node = heap.pop().ok_or("Second node is None")?;
        let w = first_node.weight + second_node.weight;
        heap.push(Box::new(Node::internal(first_node, second_node, 0, w)));
    }

    let root = heap.pop().ok_or("Heap is empty after algorithm")?;
    Ok(Tree { root, symbol_count })
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

fn create_code_table(tree: &Tree) -> Result<Vec<SymbolCode>, Box<dyn Error>> {
    let symbol_code = SymbolCode::new();
    let mut symbol_table = vec![symbol_code; TABLE_SIZE];
    walk_code_tree(&tree.root, symbol_code, &mut symbol_table);
    Ok(symbol_table)
}