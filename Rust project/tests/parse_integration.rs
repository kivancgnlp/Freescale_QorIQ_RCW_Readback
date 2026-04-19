use std::collections::HashMap;

// Inline the same bit-extraction logic from rcw.rs for testing without WASM
fn extract_bits(rcw: &[u8; 64], bit_offset: usize, width: usize) -> u64 {
    let mut value: u64 = 0;
    for i in 0..width {
        let abs_bit = bit_offset + i;
        let byte_idx = abs_bit / 8;
        let bit_in_byte = 7 - (abs_bit % 8);
        let bit = (rcw[byte_idx] >> bit_in_byte) & 1;
        value = (value << 1) | (bit as u64);
    }
    value
}

fn load_rcw() -> [u8; 64] {
    // Bytes 8..72 of PBL.bin (after 4-byte preamble + 4-byte address)
    // aa55aa55 010e0100 | <-- RCW starts here --> |
    let hex = concat!(
        "125a0000 00000000",   // bytes  8-15
        "1a1a0000 00000000",   // bytes 16-23
        "00000000 00000000",   // bytes 24-31
        "98400000 04000000",   // bytes 32-39
        "00000000 00000000",   // bytes 40-47
        "0000 0000c0da0ef7",   // bytes 48-55
        "a800 0000 00000000",  // bytes 56-63
        "00000000 00000000",   // bytes 64-71
    );
    let clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let mut rcw = [0u8; 64];
    for i in 0..64 {
        rcw[i] = u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16).unwrap();
    }
    rcw
}

#[test]
fn test_preamble_detect() {
    let bin = include_bytes!(
        "../../Sample input data/PBL.bin"
    );
    assert_eq!(&bin[0..4], &[0xAA, 0x55, 0xAA, 0x55], "preamble mismatch");
}

#[test]
fn test_sys_pll_cfg_is_zero() {
    let rcw = load_rcw();
    // Byte 0 of RCW = 0x12 = 0001 0010
    // bit_offset=0, width=2  → top 2 bits of 0x12 → 0b00 = 0
    let val = extract_bits(&rcw, 0, 2);
    assert_eq!(val, 0, "SYS_PLL_CFG expected 0, got {}", val);
}

#[test]
fn test_sys_pll_rat_is_9() {
    let rcw = load_rcw();
    // Byte 0 = 0x12 = 0001 0010
    // bits [2:6] (5 bits) = 0 1 0 0 1 = 0b01001 = 9 → 9:1 ratio
    let val = extract_bits(&rcw, 2, 5);
    assert_eq!(val, 9, "SYS_PLL_RAT expected 9, got {}", val);
}

#[test]
fn test_mem_pll_cfg() {
    let rcw = load_rcw();
    // Byte 1 of RCW = 0x5A = 0101 1010
    // bit_offset=8, width=2 → top 2 bits of byte 1 → 0b01 = 1
    let val = extract_bits(&rcw, 8, 2);
    assert_eq!(val, 1, "MEM_PLL_CFG expected 1 (higher freq ref), got {}", val);
}
