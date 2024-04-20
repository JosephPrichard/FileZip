
pub fn set_bit(num: u32, n: u32) -> u8 {
    ((1 << n) | num) as u8
}

pub fn get_bit(num: u32, n: u32) -> u8 {
    ((num >> n) & 1) as u8
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