use std::fs::{File, OpenOptions};
use std::io::{Write};
use std::mem;
use crate::bitwise;
use crate::bitwise::{get_bit, SymbolCode};
use crate::block::{FileBlock};

const BUFFER_LEN: usize = 512;
const BUFFER_BIT_LEN: u32 = (BUFFER_LEN * 8) as u32;

pub struct FileWriter {
    // the file stream to write to
    file: File,
    // a buffer storing a block to be written to the file
    buffer: [u8; BUFFER_LEN],
    // the bit position of the last write in the buffer
    bit_position: u32
}

impl FileWriter {
    pub fn new(filepath: &str) -> FileWriter {
        FileWriter {
            file: OpenOptions::new()
                .write(true)
                .append(false)
                .create(true)
                .open(filepath)
                .expect("Failed to open file for new writer"),
            buffer: [0u8; BUFFER_LEN],
            bit_position: 0
        }
    }

    fn persist_buffer(&mut self) {
        self.file.write(&self.buffer[0..((self.bit_position / 8) as usize)])
            .expect("Failed to persist buffer to file");
    }

    fn update_buffer(&mut self) {
        // check if at end of buffer: persist current buffer and start writing on a new one
        if self.bit_position >= BUFFER_BIT_LEN {
            self.persist_buffer();
            self.bit_position = 0;
            self.buffer = [0u8; BUFFER_LEN];
        }
    }

    pub fn align_to_byte(&mut self) {
        self.bit_position = ((self.bit_position + 7) / 8) * 8;
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.update_buffer();

        // write the byte directly into the buffer
        self.buffer[(self.bit_position / 8) as usize] = byte;
        self.bit_position += 8;
    }

    pub fn write_bits(&mut self, byte: u8, count: u8) {
        // write each bit individually as they might end up in different bytes in the buffer
        for i in 0..count {
            self.write_bit(get_bit(byte as u32, i as u32));
        }
    }

    pub fn write_bit(&mut self, bit: u8) {
        self.update_buffer();

        // write the bit back into the buffer
        if bit > 0 {
            let i = (self.bit_position / 8) as usize;
            self.buffer[i] = bitwise::set_bit(self.buffer[i] as u32, self.bit_position % 8);
        }

        self.bit_position += 1;
    }

    pub fn write_symbol(&mut self, symbol: &SymbolCode) {
        for i in 0..symbol.bit_len {
            self.write_bit(get_bit(symbol.encoded_symbol, i as u32));
        }
    }

    pub fn write_block(&mut self, block: &FileBlock) {
        for c in block.filename_rel.chars() {
            self.write_byte(c as u8);
        }
        self.write_byte(0);
        self.write_u64(block.tree_bit_size);
        self.write_u64(block.data_bit_size);
        self.write_u64(block.file_byte_offset);
    }

    pub fn write_u64(&mut self, num: u64) {
        let buffer: [u8; 8] = unsafe { mem::transmute(num) };
        for i in 0..8 {
            self.write_byte(buffer[i]);
        }
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        self.persist_buffer();
    }
}