pub fn u16ify(array: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([array[offset], array[offset + 1]])
}
