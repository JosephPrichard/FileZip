// Joseph Prichard
// 1/5/2023
// File IO using bit layer abstractions (read and write bits from a file)

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::io::{Read, Seek, SeekFrom};
use std::mem;
use crate::structures::{FileBlock, SymbolCode};

const BUFFER_LEN: usize = 4096;
const BUFFER_BIT_LEN: u32 = (BUFFER_LEN * 8) as u32;

// utilities for bitwise logic for io operations
pub fn set_bit(num: u32, n: u32) -> u8 {
    ((1 << n) | num) as u8
}

pub fn get_bit(num: u32, n: u32) -> u8 {
    ((num >> n) & 1) as u8
}

pub struct FileReader {
    // the file stream to read from
    file: File,
    // a buffer storing a block from the file
    buffer: [u8; BUFFER_LEN],
    // the number of bytes read from the file into the buffer
    read_size: usize,
    // the bit position of the last read in the buffer
    bit_position: u32,
    // the total number of bits read
    read_len: u64,
}

impl FileReader {
    pub fn new(filepath: &str) -> io::Result<FileReader> {
        // open the file into memory
        let mut file = File::open(filepath)?;
        // read the first buffer into memory
        let mut buffer = [0u8; BUFFER_LEN];
        let read_size = file.read(&mut buffer)?;
        // copy necessary resources into the struct
        Ok(FileReader {
            file,
            buffer,
            read_size,
            bit_position: 0,
            read_len: 0,
        })
    }

    fn update_buffer(&mut self) -> io::Result<()> {
        // at end of buffer: read a new buffer
        if self.bit_position >= BUFFER_BIT_LEN {
            self.read_size = self.file.read(&mut self.buffer)?;
            self.bit_position = 0;
        }
        Ok(())
    }

    pub fn seek(&mut self, seek_pos: u64) -> io::Result<()> {
        // seeks to location in the file for next read
        self.file.seek(SeekFrom::Start(seek_pos))?;
        // force a read to override the current buffer
        self.read_size = self.file.read(&mut self.buffer)?;
        self.bit_position = 0;
        Ok(())
    }

    pub fn read_len(&mut self) -> u64 {
        self.read_len
    }

    pub fn eof(&mut self) -> bool {
        // eof: if buffer pointer goes past read size or last buffer read was empty
        (self.bit_position > (8 * self.read_size) as u32) || self.read_size == 0
    }

    pub  fn peek_byte(&mut self) -> io::Result<u8> {
        self.update_buffer()?;
        let byte = self.buffer[(self.bit_position / 8) as usize];
        Ok(byte)
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        let byte = self.peek_byte();
        self.bit_position += 8;
        self.read_len += 8;
        byte
    }

    pub fn read_bits(&mut self, count: u8) -> io::Result<u8> {
        // read each bit individually as they might end up in different bytes in the buffer
        let mut byte = 0;
        for i in 0..count {
            if self.read_bit()? > 0 {
                byte = set_bit(byte as u32, i as u32);
            }
        }
        Ok(byte)
    }

    pub fn read_bit(&mut self) -> io::Result<u8> {
        let byte = self.peek_byte()?;
        let bit = get_bit(byte as u32, self.bit_position % 8);
        self.bit_position += 1;
        self.read_len += 1;
        Ok(bit)
    }

    pub fn read_block(&mut self) -> io::Result<FileBlock> {
        // reads string as bytes from file
        let mut filename_rel = String::from("/");
        let mut byte = self.read_byte()?;
        while byte != 0 {
            filename_rel.push(byte as char);
            byte = self.read_byte()?;
        }
        // create block and read u64 values from file into fields
        Ok(FileBlock {
            filename_rel: String::from(filename_rel),
            tree_bit_size: self.read_u64()?,
            data_bit_size: self.read_u64()?,
            file_byte_offset: self.read_u64()?,
            og_byte_size: self.read_u64()?,
        })
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        let mut buffer = [0u8; 8];
        for i in 0..8 {
            buffer[i] = self.read_byte()?;
        }
        Ok(u64::from_le_bytes(buffer))
    }
}

pub struct FileWriter {
    // the file stream to write to
    file: File,
    // a buffer storing a block to be written to the file
    buffer: [u8; BUFFER_LEN],
    // the bit position of the last write in the buffer
    bit_position: u32,
}

impl FileWriter {
    pub fn new(filepath: &str) -> io::Result<FileWriter> {
        Ok(FileWriter {
            file: OpenOptions::new()
                .write(true)
                .append(false)
                .create(true)
                .open(filepath)?,
            buffer: [0u8; BUFFER_LEN],
            bit_position: 0,
        })
    }

    fn persist_buffer(&mut self) -> io::Result<()> {
        self.file.write(&self.buffer[0..((self.bit_position / 8) as usize)])?;
        Ok(())
    }

    fn update_buffer(&mut self) -> io::Result<()> {
        // check if at end of buffer: persist current buffer and start writing on a new one
        if self.bit_position >= BUFFER_BIT_LEN {
            self.persist_buffer()?;
            self.bit_position = 0;
            self.buffer = [0u8; BUFFER_LEN];
        }
        Ok(())
    }

    pub fn align_to_byte(&mut self) -> io::Result<()> {
        self.bit_position = ((self.bit_position + 7) / 8) * 8;
        Ok(())
    }

    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.update_buffer()?;

        // write the byte directly into the buffer
        self.buffer[(self.bit_position / 8) as usize] = byte;
        self.bit_position += 8;

        Ok(())
    }

    pub fn write_bits(&mut self, byte: u8, count: u8) -> io::Result<()> {
        // write each bit individually as they might end up in different bytes in the buffer
        for i in 0..count {
            let bit = get_bit(byte as u32, i as u32);
            self.write_bit(bit)?;
        }
        Ok(())
    }

    pub fn write_bit(&mut self, bit: u8) -> io::Result<()> {
        self.update_buffer()?;

        // write the bit back into the buffer
        if bit > 0 {
            let i = (self.bit_position / 8) as usize;
            self.buffer[i] = set_bit(self.buffer[i] as u32, self.bit_position % 8);
        }

        self.bit_position += 1;
        Ok(())
    }

    pub fn write_symbol(&mut self, symbol: &SymbolCode) -> io::Result<()> {
        for i in 0..symbol.bit_len {
            let bit = get_bit(symbol.encoded_symbol, i as u32);
            self.write_bit(bit)?;
        }
        Ok(())
    }

    pub fn write_block(&mut self, block: &FileBlock) -> io::Result<()> {
        // write string with a null terminator at the end
        for c in block.filename_rel.chars() {
            self.write_byte(c as u8)?;
        }
        self.write_byte(0)?;
        // write each u64 field into the file
        self.write_u64(block.tree_bit_size)?;
        self.write_u64(block.data_bit_size)?;
        self.write_u64(block.file_byte_offset)?;
        self.write_u64(block.og_byte_size)?;
        Ok(())
    }

    pub fn write_u64(&mut self, num: u64) -> io::Result<()> {
        let buffer: [u8; 8] = unsafe { mem::transmute(num) };
        for i in 0..8 {
            self.write_byte(buffer[i])?;
        }
        Ok(())
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        if let Err(e) = self.persist_buffer() {
            panic!("Fatal: failed to write the buffer to file when dropping: {}", e.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitwise() {
        // little endian left to right ordering
        let mut num = 0b01011;
        let bits = [1, 1, 0, 1, 0];

        for (i, bit) in bits.iter().enumerate() {
            assert_eq!(get_bit(num, i as u32), *bit);
        }

        num = set_bit(num, 2) as u32;
        assert_eq!(num, 0b01111);
    }
}