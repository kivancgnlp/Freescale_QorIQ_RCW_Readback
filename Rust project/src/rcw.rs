/// Extracts `width` bits starting at `bit_offset` from a 64-byte big-endian
/// bit array (MSB of byte 0 = bit 0).
pub fn extract_bits(rcw: &[u8; 64], bit_offset: usize, width: usize) -> u64 {
    assert!(width <= 64, "field width must be ≤ 64");
    assert!(
        bit_offset + width <= 512,
        "field [{}..{}] exceeds 512-bit RCW",
        bit_offset,
        bit_offset + width
    );

    let mut value: u64 = 0;
    for i in 0..width {
        let abs_bit = bit_offset + i;
        let byte_idx = abs_bit / 8;
        let bit_in_byte = 7 - (abs_bit % 8); // MSB = bit 7 of the byte
        let bit = (rcw[byte_idx] >> bit_in_byte) & 1;
        value = (value << 1) | (bit as u64);
    }
    value
}
