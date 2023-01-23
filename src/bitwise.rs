// Joseph Prichard
// 1/5/2023
// Bytewise and bitwise utilities for symbols to be compressed

#[derive(Clone, Copy)]
pub struct SymbolCode {
    pub plain_symbol: u8,
    pub encoded_symbol: u32,
    pub bit_len: u8
}

impl SymbolCode {
    pub fn new() -> SymbolCode {
        SymbolCode { plain_symbol: 0, encoded_symbol: 0, bit_len: 0 }
    }

    pub fn append_bit(&self, bit: u32) -> SymbolCode {
        SymbolCode {
            plain_symbol: self.plain_symbol,
            encoded_symbol: self.encoded_symbol ^ (bit << self.bit_len),
            bit_len: self.bit_len + 1
        }
    }
}

pub fn get_bit(num: u32, n: u32) -> u8 {
    ((num >> n) & 1) as u8
}

pub fn set_bit(num: u32, n: u32) -> u8 {
    ((1 << n) | num) as u8
}