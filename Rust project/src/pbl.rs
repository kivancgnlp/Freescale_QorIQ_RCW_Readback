/// Extracts the 64-byte (512-bit) RCW payload from a PBL binary.
///
/// PBL layout:
///   Bytes 0–3  : 0xAA 0x55 0xAA 0x55  — preamble magic
///   Bytes 4–7  : destination address   — big-endian u32 (e.g. 0x010E0100)
///   Bytes 8–71 : 64-byte RCW payload
///   Bytes 72+  : PBI instructions, terminated by end command 0x08138040 + 4-byte CRC
pub fn extract_rcw(data: &[u8]) -> Result<[u8; 64], String> {
    const PREAMBLE: [u8; 4] = [0xAA, 0x55, 0xAA, 0x55];
    const HEADER_LEN: usize = 8; // preamble (4) + address (4)
    const RCW_LEN: usize = 64;

    if data.len() < HEADER_LEN + RCW_LEN {
        return Err(format!(
            "File too small ({} bytes). A valid PBL binary must be at least {} bytes \
             (4-byte preamble + 4-byte destination address + 64-byte RCW payload).",
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

pub struct CrcResult {
    pub stored: u32,
    pub computed: u32,
    pub ok: bool,
}

/// Scans for the PBL end command (0x08138040) and validates the 4-byte CRC that follows it.
/// CRC algorithm: CRC32/MPEG-2 (poly 0x04C11DB7, init 0xFFFFFFFF, non-reflected, no final XOR)
/// covering all bytes from the start of the PBL image through the end of the command word.
/// Returns None if no end command is found in the supplied data.
pub fn check_pbl_crc(data: &[u8]) -> Option<CrcResult> {
    const END_CMD: [u8; 4] = [0x08, 0x13, 0x80, 0x40];

    let mut i = 0;
    while i + 8 <= data.len() {
        if data[i..i + 4] == END_CMD {
            let stored = u32::from_be_bytes([data[i+4], data[i+5], data[i+6], data[i+7]]);
            let computed = crc32_mpeg2(&data[..i + 4]);
            return Some(CrcResult { stored, computed, ok: stored == computed });
        }
        i += 4;
    }
    None
}

fn crc32_mpeg2(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= (byte as u32) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04C1_1DB7
            } else {
                crc << 1
            };
            crc &= 0xFFFF_FFFF;
        }
    }
    crc
}
