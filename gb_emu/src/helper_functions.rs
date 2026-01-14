pub fn concat_2_bytes(byte1: u8, byte2: u8) -> u16 {
    ((byte1 as u16) << 8) | byte2 as u16
}

/// gets a chunk of bits in a byte specified by the leftmost and rightmost (inclusive) bits desired.
pub fn get_bitfield(mut byte: u8, left: u8, right: u8) -> u8 {
    byte = (byte << (7 - left)) >> (7 - left); // zero out left bits
    byte >>= right; // shift remaining bits to the far right of the return val
    byte
}

/// Maybe we'll make this into an actual log some day but for now, we'll just print it out.
pub fn log(message: &str) {
    println!("Log Message: {message}")
}

#[cfg(test)]
mod test {
    use crate::helper_functions::get_bitfield;

    #[test]
    fn test_get_bitfield() {
        let start = 0b11001010u8;
        let expected = 0b0000_1001u8;
        let result = get_bitfield(start, 6, 3);

        assert_eq!(expected, result)
    }
}
