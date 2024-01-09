// Joseph Prichard
// 1/5/2023
// Byte-by-byte file compressor

use std::collections::{BinaryHeap};
use std::{fs, io};
use std::path::{Path};
use std::time::Instant;
use rayon::prelude::*;
use crate::bitwise::SymbolCode;
use crate::data::{CodeBook, FileBitSize, FileBlock, TABLE_SIZE};
use crate::{charset, parallelism};
use crate::tree::{Tree, CodeTree};
use crate::read::{BitwiseReader, FileReader};
use crate::{data, utils};
use crate::write::{BitwiseWriter, FileWriter};

pub fn archive_dir(input_entry: &[String], multithreaded: bool) -> io::Result<()> {
    let now = Instant::now();

    let mut blocks = get_file_blocks(input_entry)?;

    parallelism::configure_thread_pool(multithreaded, blocks.len());

    let mut code_books = create_code_books(&mut blocks)?;

    let archive_filename = String::from(&input_entry[0]) + ".zipr";
    let writer = &mut FileWriter::new(&archive_filename)?;
    writer.write_u64(charset::SIG)?;

    write_block_headers(writer, &mut code_books)?;
    compress_files(writer, &code_books)?;

    let elapsed = now.elapsed();
    println!("Finished zipping in {:.2?}", elapsed);
    data::list_file_blocks(&blocks);

    Ok(())
}

fn get_file_blocks(entries: &[String]) -> io::Result<Vec<FileBlock>> {
    let mut blocks = vec![];
    for entry in entries {
        let path = Path::new(entry);
        walk_path(path.parent().expect("Failed to get parent path"), path, &mut blocks)?;
    }
    Ok(blocks)
}

fn walk_path(base_path: &Path, path: &Path, blocks: &mut Vec<FileBlock>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            walk_path(&base_path, &path, blocks)?;
        }
        Ok(())
    } else {
        let filename_abs = &String::from(path.to_str().unwrap());
        let filename_rel = &String::from(path
            .strip_prefix(base_path)
            .unwrap()
            .to_str()
            .unwrap());
        let size = utils::dir_entry_size(&path);
        let block = FileBlock::from_fs_metadata(filename_rel, filename_abs, size);
        blocks.push(block);
        Ok(())
    }
}

fn create_code_books(blocks: &mut [FileBlock]) -> io::Result<Vec<CodeBook>> {
    // create code books, this operation can be parallelized because it only reads
    blocks.into_par_iter()
        .map(|block| create_code_book(block))
        .collect()
}

fn create_code_book(block: &mut FileBlock) -> io::Result<CodeBook> {
    let reader = &mut FileReader::new(&block.filename_abs)?;
    let freq_table = create_freq_table(reader)?;
    let tree = create_code_tree(&freq_table);
    let symbol_table = Box::new(create_code_table(&tree));
    block.fbs = calc_file_bit_sizes(&symbol_table, &freq_table);

    // create the prepared file block containing the code book
    Ok(CodeBook {
        symbol_table,
        tree,
        block: block.clone(),
    })
}

fn calc_file_bit_sizes(symbol_table: &[SymbolCode; TABLE_SIZE], freq_table: &[u64]) -> FileBitSize {
    let mut fbs = FileBitSize { tree_bit_size: 0, data_bit_size: 0 };
    // calculate the bit size for the file block for compressed data and for tree
    let mut char_count = 0;
    for i in 0..TABLE_SIZE {
        let freq = freq_table[i];
        fbs.data_bit_size += freq * (symbol_table[i].bit_len as u64);
        if freq > 0 {
            char_count += 1;
        }
    }
    fbs.tree_bit_size += 10 * char_count - 1;
    fbs
}

fn write_block_headers(writer: &mut dyn BitwiseWriter, code_books: &mut [CodeBook]) -> io::Result<()> {
    // calculate the total block size for the header, including the grp sep byte
    let mut header_size = 1;
    for code_book in code_books.iter_mut() {
        // header size plus an additional rec sep byte
        let block = &code_book.block;
        header_size += block.get_header_size() + 1;
    }

    let mut total_offset = 0;
    for code_book in code_books.iter_mut() {
        // write record sep to identify start of record
        writer.write_byte(charset::REC_SEP)?;

        let fbs = &code_book.block.fbs;
        code_book.block.file_byte_offset = header_size + total_offset;
        total_offset += 1 + (fbs.data_bit_size + fbs.tree_bit_size) / 8;

        writer.write_block(&code_book.block)?;
    }
    // write group sep after headers are complete
    writer.write_byte(charset::GRP_SEP)?;

    Ok(())
}

fn compress_files(writer: &mut dyn BitwiseWriter, code_books: &[CodeBook]) -> io::Result<()> {
    for code_book in code_books {
        write_tree(writer, &code_book.tree.root)?;
        let reader = &mut FileReader::new(&code_book.block.filename_abs)?;
        compress(reader, writer, code_book.symbol_table.as_ref())?;
        writer.align_to_byte()?;
    }
    Ok(())
}

fn write_tree(writer: &mut dyn BitwiseWriter, tree: &Box<Tree>) -> io::Result<()> {
    if tree.is_leaf() {
        writer.write_bit(1)?;
        writer.write_bits(tree.plain_symbol, 8)?;
        Ok(())
    } else {
        writer.write_bit(0)?;
        let left = tree.left.as_ref().expect("Expected left node to be Some");
        write_tree(writer, left)?;
        let right = tree.right.as_ref().expect("Expected right node to be Some");
        write_tree(writer, right)
    }
}

fn compress(reader: &mut dyn BitwiseReader, writer: &mut dyn BitwiseWriter, symbol_table: &[SymbolCode]) -> io::Result<()> {
    while !reader.eof() {
        let byte = reader.read_aligned_byte()?;
        let symbol = &symbol_table[byte as usize];
        writer.write_symbol(symbol)?;
    }
    Ok(())
}

fn create_freq_table(reader: &mut dyn BitwiseReader) -> io::Result<[u64; TABLE_SIZE]> {
    let mut freq_table = [0u64; TABLE_SIZE];

    // iterate through each byte in the file and increment count
    while !reader.eof() {
        let byte = reader.read_aligned_byte()?;
        freq_table[byte as usize] += 1;
    }

    Ok(freq_table)
}

fn create_code_tree(freq_table: &[u64]) -> CodeTree {
    let mut heap = BinaryHeap::new();

    // add the frequency table nodes to priority queue
    let mut symbol_count = 0;
    for i in 0..TABLE_SIZE {
        let freq = freq_table[i];
        if freq != 0 {
            heap.push(Box::new(Tree::leaf(i as u8, freq)));
            symbol_count += 1;
        }
    }

    // huffman coding algorithm
    while heap.len() >= 2 {
        // invariant: the heap should never have 1 or 0 elements at this point
        let first_node = heap.pop().expect("First node is None");
        let second_node = heap.pop().expect("Second node is None");
        let w = first_node.weight + second_node.weight;
        heap.push(Box::new(Tree::internal(first_node, second_node, 0, w)));
    }

    // invariant: the heap should not be empty after the huffman coding algorithm is finished
    let root = heap.pop().expect("Heap is empty after algorithm");
    CodeTree { root, symbol_count }
}

fn walk_code_tree(node: &Box<Tree>, mut symbol_code: SymbolCode, symbol_table: &mut [SymbolCode]) {
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

fn create_code_table(tree: &CodeTree) -> [SymbolCode; TABLE_SIZE] {
    let symbol_code = SymbolCode::new();
    let mut symbol_table = [symbol_code; TABLE_SIZE];
    walk_code_tree(&tree.root, symbol_code, &mut symbol_table);
    symbol_table
}