// Joseph Prichard
// 1/5/2023
// File reading and writing utilities

use std::error::Error;
use std::fs::File;
use std::io::Read;
use crate::bitwise;

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
    bit_position: u32
}

impl FileReader {
    pub fn new(filepath: &str) -> Result<FileReader, Box<dyn Error>> {
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
            bit_position: 0
        })
    }

    pub fn eof(&mut self) -> bool {
        // eof: if buffer pointer goes past read size or last buffer read was empty
        (self.bit_position > (8 * self.read_size) as u32) || self.read_size == 0
    }

    fn update_buffer(&mut self) -> Result<(), Box<dyn Error>> {
        // at end of buffer: read a new buffer
        if self.bit_position > BUFFER_BIT_LEN {
            self.read_size = self.file.read(&mut self.buffer)?;
            self.bit_position = 0;
        }
        Ok(())
    }

    fn load_byte(&mut self) -> Result<u8, Box<dyn Error>> {
        self.update_buffer()?;
        Ok(self.buffer[(self.bit_position / 8) as usize])
    }

    pub fn read_byte(&mut self) -> Result<u8, Box<dyn Error>> {
        let byte = self.load_byte()?;
        self.bit_position += 8;
        Ok(byte)
    }

    pub fn read_bits(&mut self, count: u8) -> Result<u8, Box<dyn Error>> {
        // read each bit individually as they might end up in different bytes in the buffer
        let mut byte = 0;
        for i in 0..count {
            if self.read_bit()? > 0 {
                byte = bitwise::set_bit(byte as u32, i as u32);
            }
        }
        Ok(byte)
    }

    pub fn read_bit(&mut self) -> Result<u8, Box<dyn Error>> {
        let byte = self.load_byte()?;
        let bit = bitwise::get_bit(byte as u32, self.bit_position % 8);
        self.bit_position += 1;
        Ok(bit)
    }
}
