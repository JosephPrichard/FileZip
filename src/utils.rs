pub fn get_size_of<T>(_: T) -> usize {
    std::mem::size_of::<T>()
}

pub const fn str_to_u64(str: &str) -> u64 {
    let mut buffer = [0u8; 8];
    let mut i = 0;
    while i < str.len() && i < 8 {
        buffer[i] = str.as_bytes()[i];
        i += 1;
    }
    u64::from_le_bytes(buffer)
}