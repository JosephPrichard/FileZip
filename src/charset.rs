// Joseph Prichard
// 1/5/2023
// Char code globals for file format

use crate::utils::str_to_u64;

pub const REC_SEP: u8 = 0x1E;
pub const GRP_SEP: u8 = 0x1D;
pub const SIG: u64 = str_to_u64("zipper");