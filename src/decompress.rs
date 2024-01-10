// Joseph Prichard
// 1/5/2023
// Bit-by-bit file decompressor

use std::{fs, io};
use std::path;
use std::path::{Path};
use std::time::Instant;
use rayon::prelude::*;
use crate::block::{FileBlock};
use crate::compress::{GRP_SEP, SIG};
use crate::parallelism;
use crate::read::{BitReader, FileReader};
use crate::write::BitWriter;
use crate::tree::Tree;
use crate::utils;
use crate::write::FileWriter;

pub fn unarchive_zip(archive_filepath: &str, multithreaded: bool) -> io::Result<()> {
    let now = Instant::now();

    let output_dir = utils::get_no_ext(archive_filepath);
    fs::create_dir_all(&output_dir)?;

    let blocks_reader = &mut FileReader::new(archive_filepath)?;
    let blocks = get_file_blocks(blocks_reader)?;

    parallelism::configure_thread_pool(multithreaded, blocks.len())?;
    decompress_files(&blocks, archive_filepath, &output_dir)?;

    let elapsed = now.elapsed();
    println!("Finished unzipping in {:.2?}", elapsed);
    Ok(())
}

pub fn get_file_blocks(reader: &mut dyn BitReader) -> io::Result<Vec<FileBlock>> {
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

fn decompress_files(blocks: &[FileBlock], archive_filepath: &str, output_dir: &str) -> io::Result<()> {
    // decompress each file, this can be parallelized because each function call writes to a different file
    blocks.par_iter()
        .map(|block| decompress_file(block, archive_filepath, output_dir))
        .collect()
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

// read the contents of a compressed archive and write into a decompressed stream
fn decompress(block: &FileBlock, reader: &mut dyn BitReader, writer: &mut dyn BitWriter) -> io::Result<()> {
    // read from the main archive: jumping to the data segment
    reader.seek((utils::get_size_of(SIG) as u64) + block.file_byte_offset)?;

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
fn read_tree(reader: &mut dyn BitReader) -> io::Result<Box<Tree>> {
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
fn decompress_symbol(reader: &mut dyn BitReader, writer: &mut dyn BitWriter, node: &Box<Tree>) -> io::Result<()> {
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
