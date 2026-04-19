/// Extracts the 64-byte (512-bit) RCW payload from a PBL binary.
///
/// PBL layout:
///   Bytes 0–3  : 0xAA 0x55 0xAA 0x55  — preamble magic
///   Bytes 4–7  : destination address   — big-endian u32 (e.g. 0x010E0100)
///   Bytes 8–71 : 64-byte RCW payload   ← returned by this function
///   Bytes 72+  : PBI instructions, terminated by 0x08138040
pub fn extract_rcw(data: &[u8]) -> Result<[u8; 64], String> {
    const PREAMBLE: [u8; 4] = [0xAA, 0x55, 0xAA, 0x55];
    const HEADER_LEN: usize = 8; // preamble (4) + address (4)
    const RCW_LEN: usize = 64;

    if data.len() < HEADER_LEN + RCW_LEN {
        return Err(format!(
            "File too short: {} bytes (need at least {})",
            data.len(),
            HEADER_LEN + RCW_LEN
        ));
    }

    if data[0..4] != PREAMBLE {
        return Err(format!(
            "Invalid preamble: {:02X} {:02X} {:02X} {:02X} (expected AA 55 AA 55)",
            data[0], data[1], data[2], data[3]
        ));
    }

    let mut rcw = [0u8; RCW_LEN];
    rcw.copy_from_slice(&data[HEADER_LEN..HEADER_LEN + RCW_LEN]);
    Ok(rcw)
}
