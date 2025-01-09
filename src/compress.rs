// Joseph Prichard
// 1/5/2023
// Byte-by-byte file compressor and decompressor

use std::collections::BinaryHeap;
use std::thread::available_parallelism;
use std::{fs, io, path};
use std::path::Path;
use std::time::Instant;
use rayon::prelude::*;
use rayon::ThreadPool;
use crate::structures::{FileBlock, SymbolCode, Tree};
use crate::bitwise_io::{FileReader, FileWriter};

pub const TABLE_SIZE: usize = 256;
pub const REC_SEP: u8 = 0x1E;
pub const GRP_SEP: u8 = 0x1D;
pub const SIG: u64 = str_to_u64("zipper");

pub fn configure_thread_pool(multithreaded: bool, file_count: usize) -> io::Result<ThreadPool> {
    let threads = if multithreaded {
        let cores = available_parallelism()?.get();
        file_count.min(cores)
    } else {
        1
    };

    println!("Running with {} threads", threads);
    let tp = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Failed to configure thread pool");
    Ok(tp)
}

pub const fn str_to_u64(str: &str) -> u64 {
    let mut buffer = [0u8; 8];
    let mut i = 0;
    // converts a string to a buffer
    while i < str.len() && i < 8 {
        buffer[i] = str.as_bytes()[i];
        i += 1;
    }
    // converts a buffer to a u64
    u64::from_le_bytes(buffer)
}

pub fn archive_dir(input_entry: &[String], multithreaded: bool) -> io::Result<Vec<FileBlock>> {
    let labels = get_file_labels(input_entry)?;

    let now = Instant::now();

    let tp = configure_thread_pool(multithreaded, labels.len())?;
    let code_books = create_code_books(&labels, &tp)?;

    let blocks = create_file_blocks(&code_books);

    let archive_filename = fs::canonicalize(String::from(&input_entry[0]) + ".zipr")?;
    let archive_filename = archive_filename.to_str().unwrap();

    let writer = &mut FileWriter::new(archive_filename)?;
    writer.write_u64(SIG)?;
    write_block_headers(writer, &blocks)?;
    compress_files(writer, &code_books)?;

    let elapsed = now.elapsed();
    println!("Finished zipping in {:.2?}", elapsed);
    println!("Wrote archive to: {}", &archive_filename);

    Ok(blocks)
}

pub fn list_file_blocks(blocks: &[FileBlock]) {
    println!("{:>15}\t\t{:>15}\t\t{:>8}\t\t{:25}", "compressed", "uncompressed", "ratio", "uncompressed_name");

    for block in blocks {
        let total_byte_size = (block.data_bit_size + block.tree_bit_size) / 8;
        let ratio_str = format!("{:.2}%", (total_byte_size as f64) / (block.og_byte_size as f64) * 100.0);

        println!("{:>15}\t\t{:>15}\t\t{:>8}\t\t{:25}", total_byte_size, block.og_byte_size, &ratio_str, &block.filename_rel);
    }
    println!();
}

struct FileLabel {
    filename_abs: String,
    filename_rel: String,
    size: u64,
}

// get file system metadata for the files to be compressed
fn get_file_labels(entries: &[String]) -> io::Result<Vec<FileLabel>> {
    let mut labels = vec![];
    for entry in entries {
        let path = Path::new(entry);
        let base_path = path.parent().unwrap_or_else(|| Path::new(""));
        walk_path(base_path, path, &mut labels)?;
    }
    Ok(labels)
}

fn walk_path(base_path: &Path, path: &Path, labels: &mut Vec<FileLabel>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            walk_path(&base_path, &path, labels)?;
        }
        Ok(())
    } else {
        // invariant: a valid path is also a valid string in this context
        let filename_abs = String::from(path.to_str()
            .expect("Expected file path to be valid string"));

        // invariant: the base path must be a valid prefix of the path and an empty string is always a valid prefix
        let filename_rel = String::from(path
            .strip_prefix(base_path)
            .expect("Expected base path to be a valid prefix of lower path")
            .to_str()
            .expect("Expected file path to be valid string"));

        let size = dir_entry_size(&path);
        let file = FileLabel { filename_abs, filename_rel, size };
        labels.push(file);
        Ok(())
    }
}

pub fn dir_entry_size(path: &Path) -> u64 {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Can't read directory") {
            let entry = entry.expect("Entry is invalid");
            let path = entry.path();
            size += dir_entry_size(&path);
        }
    } else {
        size += path.metadata().expect("Can't get metadata").len();
    }
    size
}

// a codebook is an instruction set specifying what to compress and how it should be done
struct CodeBook<'a> {
    label: &'a FileLabel,
    symbol_table: Box<[SymbolCode; TABLE_SIZE]>,
    tree: CodeTree,
    freq_table: Box<[u64; TABLE_SIZE]>,
}

fn create_code_books<'a>(labels: &'a [FileLabel], tp: &ThreadPool) -> io::Result<Vec<CodeBook<'a>>> {
    // create code books, this operation can be parallelized because it only reads
    tp.install(|| {
        labels.into_par_iter()
            .map(|label| create_code_book(label))
            .collect()
    })
}

// create a codebook from the intermediate file block argument
fn create_code_book(label: &FileLabel) -> io::Result<CodeBook> {
    let reader = &mut FileReader::new(&label.filename_abs)?;
    let freq_table = create_freq_table(reader)?;
    let tree = create_code_tree(freq_table.as_ref());
    let symbol_table = create_code_table(&tree);
    Ok(CodeBook { label, symbol_table, tree, freq_table })
}

fn create_freq_table(reader: &mut FileReader) -> io::Result<Box<[u64; TABLE_SIZE]>> {
    let mut freq_table = [0u64; TABLE_SIZE];
    // iterate through each byte in the file and increment count
    while !reader.eof() {
        let byte = reader.read_byte()?;
        freq_table[byte as usize] += 1;
    }
    Ok(Box::new(freq_table))
}

pub struct CodeTree {
    pub root: Box<Tree>,
    pub symbol_count: u32,
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
        let first_node = heap.pop()
            .expect("Expected first node to be Some after checking length");
        let second_node = heap.pop()
            .expect("Expected second node to be Some after checking length");
        let w = first_node.weight + second_node.weight;
        heap.push(Box::new(Tree::internal(first_node, second_node, 0, w)));
    }

    // invariant: the heap should not be empty after the huffman coding algorithm is finished
    let root = heap.pop()
        .expect("Expected heap to have at least one element after huffman coding algorithm");
    CodeTree { root, symbol_count }
}

fn create_code_table(tree: &CodeTree) -> Box<[SymbolCode; TABLE_SIZE]> {
    let symbol_code = SymbolCode::new();
    let mut symbol_table = [symbol_code; TABLE_SIZE];
    walk_code_tree(&tree.root, symbol_code, &mut symbol_table);
    Box::new(symbol_table)
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

// create the file blocks to be put into the archive - missing the offset this is calculated at write time
fn create_file_blocks(code_books: &[CodeBook]) -> Vec<FileBlock> {
    let mut blocks = vec![];
    for code_book in code_books {
        let mut tree_bit_size = 0u64;
        let mut data_bit_size = 0u64;

        // calculate the bit size for the file block for compressed data and for tree
        let mut char_count = 0;
        for i in 0..TABLE_SIZE {
            let freq = code_book.freq_table[i];
            data_bit_size += freq * (code_book.symbol_table[i].bit_len as u64);
            if freq > 0 {
                char_count += 1;
            }
        }
        tree_bit_size += 10 * char_count - 1;

        let block = FileBlock {
            filename_rel: String::from(&code_book.label.filename_rel),
            file_byte_offset: 0,
            og_byte_size: code_book.label.size,
            tree_bit_size,
            data_bit_size,
        };
        blocks.push(block);
    }
    blocks
}

fn write_block_headers(writer: &mut FileWriter, blocks: &[FileBlock]) -> io::Result<()> {
    // calculate the total block size for the header, including the grp sep byte
    let mut header_size = 1;
    for block in blocks {
        // header size plus an additional rec sep byte
        let block = &block;
        header_size += block.get_header_size() + 1;
    }

    let mut total_offset = 0;
    for block in blocks {
        // write record sep to identify start of record
        writer.write_byte(REC_SEP)?;

        // calculate the offset of the compressed data using values from all previous file blocks
        let mut block = block.clone();
        block.file_byte_offset = header_size + total_offset;
        total_offset += 1 + (block.data_bit_size + block.tree_bit_size) / 8;

        writer.write_block(&block)?;
    }
    // write group sep after headers are complete
    writer.write_byte(GRP_SEP)?;
    Ok(())
}

fn compress_files(writer: &mut FileWriter, code_books: &[CodeBook]) -> io::Result<()> {
    for code_book in code_books {
        write_tree(writer, &code_book.tree.root)?;

        let reader = &mut FileReader::new(&code_book.label.filename_abs)?;
        while !reader.eof() {
            let byte = reader.read_byte()?;
            let symbol = &code_book.symbol_table[byte as usize];
            writer.write_symbol(symbol)?;
        }

        writer.align_to_byte()?;
    }
    Ok(())
}

fn write_tree(writer: &mut FileWriter, tree: &Box<Tree>) -> io::Result<()> {
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

pub fn unarchive_zip(archive_filepath: &str, multithreaded: bool) -> io::Result<()> {
    let output_dir = strip_ext(archive_filepath);
    fs::create_dir_all(&output_dir)?;

    let now = Instant::now();

    let blocks_reader = &mut FileReader::new(archive_filepath)?;
    let blocks = get_file_blocks(blocks_reader)?;

    let tp = configure_thread_pool(multithreaded, blocks.len())?;
    decompress_files(&blocks, archive_filepath, &output_dir, &tp)?;

    let elapsed = now.elapsed();
    println!("Finished unzipping in {:.2?}", elapsed);
    Ok(())
}

pub fn strip_ext(path: &str) -> String {
    Path::new(path)
        .with_extension("")
        .display()
        .to_string()
}

pub fn get_file_blocks(reader: &mut FileReader) -> io::Result<Vec<FileBlock>> {
    if reader.read_u64()? != SIG {
        return Err(io::Error::new(
            io::ErrorKind::Other, "Cannot read from an invalid zipr file"));
    }
    // iterate through headers until the file separator byte is found or eof
    let mut blocks = vec![];
    while !reader.eof() {
        let sep = reader.read_byte()?;
        if sep == GRP_SEP {
            break;
        }
        let block = reader.read_block()?;
        blocks.push(block);
    }
    Ok(blocks)
}

fn decompress_files(blocks: &[FileBlock], archive_filepath: &str, output_dir: &str, tp: &ThreadPool) -> io::Result<()> {
    // decompress each file, this can be parallelized because each function call writes to a different file
    tp.install(|| {
        blocks.par_iter()
            .map(|block| decompress_file(block, archive_filepath, output_dir))
            .collect()
    })
}

fn decompress_file(block: &FileBlock, archive_filepath: &str, output_dir: &str) -> io::Result<()> {
    let unarchived_filename = &format!("{}{}{}", output_dir, path::MAIN_SEPARATOR, &block.filename_rel);
    if let Some(unarchived_parent) = Path::new(unarchived_filename).parent() {
        fs::create_dir_all(unarchived_parent)?;
    }

    let writer = &mut FileWriter::new(unarchived_filename)?;
    let reader = &mut FileReader::new(archive_filepath)?;
    decompress(&block, reader, writer)
}

pub fn sizeof<T>(_: T) -> usize {
    std::mem::size_of::<T>()
}

// read the contents of a compressed archive and write into a decompressed stream
fn decompress(block: &FileBlock, reader: &mut FileReader, writer: &mut FileWriter) -> io::Result<()> {
    // read from the main archive: jumping to the data segment
    reader.seek((sizeof(SIG) as u64) + block.file_byte_offset)?;

    let root = read_tree(reader)?;

    // decompress each symbol in data segment, stopping at the end
    let start_read_len = reader.read_len() as i64;
    while !reader.eof() {
        let read_len = reader.read_len() as i64;
        if (read_len - start_read_len) > (block.data_bit_size as i64 - 8) {
            break;
        }
        decompress_symbol(reader, writer, &root)?;
    }
    Ok(())
}

// read the tree from a compressed archive
fn read_tree(reader: &mut FileReader) -> io::Result<Box<Tree>> {
    let bit = reader.read_bit()?;
    if bit == 1 {
        // read 8 unaligned bits
        let symbol = reader.read_bits(8)?;
        Ok(Box::new(Tree::leaf(symbol, 0)))
    } else {
        let left = read_tree(reader)?;
        let right = read_tree(reader)?;
        Ok(Box::new(Tree::internal(left, right, 0, 0)))
    }
}

// read the next symbol from the compressed archived and write it into a decompressed stream using the codebook tree
fn decompress_symbol(reader: &mut FileReader, writer: &mut FileWriter, node: &Box<Tree>) -> io::Result<()> {
    if node.is_leaf() {
        writer.write_byte(node.plain_symbol)?;
        Ok(())
    } else {
        let bit = reader.read_bit()?;
        // invariant: a non-leaf should have left and right nodes in a full tree
        if bit == 0 {
            let left = node.left.as_ref().expect("Expected left node to be Some");
            decompress_symbol(reader, writer, left)
        } else {
            let right = node.right.as_ref().expect("Expected right node to be Some");
            decompress_symbol(reader, writer, right)
        }
    }
}

mod tests {
    use std::{collections::HashMap, fs};
    use crate::compress::{archive_dir, unarchive_zip};

    #[test]
    fn test_compress_directory() {
        let input_path = String::from("./test/files");

        let mut dir_data = HashMap::new();
        for entry in fs::read_dir(&input_path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                continue
            }
            let file_data = fs::read_to_string(&path)
                .expect(&format!("Cannot read file at path {}", path.to_str().unwrap()));

            let relative_path = path.strip_prefix(&input_path).unwrap().to_owned();
            dir_data.insert(relative_path.clone(), file_data);
        }
        println!("Directory files {:?}", dir_data.keys());

        archive_dir(&[input_path], false).unwrap();
        unarchive_zip("./test/files.zipr", false).unwrap();

        let output_path = "./test/files/files";
        for entry in fs::read_dir(output_path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                continue
            }
            let file_data = fs::read_to_string(&path)
                .expect(&format!("Cannot read at file path {}", path.to_str().unwrap()));

            let relative_path = path.strip_prefix(&output_path).unwrap();
            let other_file_data = dir_data.get(relative_path)
                .expect(&format!("Cannot find path in map {}", path.to_str().unwrap()));

            if file_data != *other_file_data {
                panic!("File data for file path is different: {}", path.to_str().unwrap())
            }
        }

        fs::remove_dir_all("./test/files/files").unwrap();
    }
}