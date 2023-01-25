// Joseph Prichard
// 1/5/2023
// File reading and writing utilities

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use crate::bitwise;
use crate::block::{FileBlock};

const BUFFER_LEN: usize = 512;
const BUFFER_BIT_LEN: u32 = (BUFFER_LEN * 8) as u32;

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
    read_len: u64
}

impl FileReader {
    pub fn new(filepath: &str) -> FileReader {
        // open the file into memory
        let mut file = File::open(filepath)
            .expect("Failed to create file for new reader");
        // read the first buffer into memory
        let mut buffer = [0u8; BUFFER_LEN];
        let read_size = file.read(&mut buffer)
            .expect("Failed to read buffer for new reader");
        // copy necessary resources into the struct
        FileReader {
            file,
            buffer,
            read_size,
            bit_position: 0,
            read_len: 0
        }
    }

    pub fn seek_from_start(&mut self, seek_pos: u64) {
        // seeks to location in the file for next read
        self.file.seek(SeekFrom::Start(seek_pos))
            .expect("Failed to seek to location in reader");
        // force a read to override the current buffer
        self.read_size = self.file.read(&mut self.buffer)
            .expect("Failed to read buffer in update");
        self.bit_position = 0;
    }

    pub fn read_len(&mut self) -> u64 {
        self.read_len
    }

    pub fn eof(&mut self) -> bool {
        // eof: if buffer pointer goes past read size or last buffer read was empty
        (self.bit_position > (8 * self.read_size) as u32) || self.read_size == 0
    }

    fn update_buffer(&mut self) {
        // at end of buffer: read a new buffer
        if self.bit_position >= BUFFER_BIT_LEN {
            self.read_size = self.file.read(&mut self.buffer)
                .expect("Failed to read buffer in update");
            self.bit_position = 0;
        }
    }

    pub fn view_byte(&mut self) -> u8 {
        self.update_buffer();
        self.buffer[(self.bit_position / 8) as usize]
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.view_byte();
        self.bit_position += 8;
        self.read_len += 8;
        byte
    }

    pub fn read_bits(&mut self, count: u8) -> u8 {
        // read each bit individually as they might end up in different bytes in the buffer
        let mut byte = 0;
        for i in 0..count {
            if self.read_bit() > 0 {
                byte = bitwise::set_bit(byte as u32, i as u32);
            }
        }
        byte
    }

    pub fn read_bit(&mut self) -> u8 {
        let byte = self.view_byte();
        let bit = bitwise::get_bit(byte as u32, self.bit_position % 8);
        self.bit_position += 1;
        self.read_len += 1;
        bit
    }

    pub fn read_block(&mut self) -> FileBlock {
        // reads string as bytes from file
        let mut filename_rel = String::new();
        let mut byte = self.read_byte();
        while byte != 0 {
            filename_rel.push(byte as char);
            byte = self.read_byte();
        }
        // create block and read u64 values from file into fields
        let mut block = FileBlock::new(&filename_rel, "");
        block.tree_bit_size = self.read_u64();
        block.data_bit_size = self.read_u64();
        block.file_byte_offset = self.read_u64();
        block.original_byte_size = self.read_u64();
        block
    }

    pub fn read_u64(&mut self) -> u64 {
        let mut buffer = [0u8; 8];
        for i in 0..8 {
            buffer[i] = self.read_byte();
        }
        u64::from_le_bytes(buffer)
    }
}
