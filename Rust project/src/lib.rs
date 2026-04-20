mod config;
mod pbl;
mod rcw;

use serde::Serialize;
use wasm_bindgen::prelude::*;

// Embed processor configs at compile time
const P2041_XML: &str = include_str!("../config/p2041_rcw.xml");
const P3041_XML: &str = include_str!("../config/p3041_rcw.xml");
const P4080_XML: &str = include_str!("../config/p4080_rcw.xml");
const P5020_XML: &str = include_str!("../config/p5020_rcw.xml");
const P5040_XML: &str = include_str!("../config/p5040_rcw.xml");
const T1024_XML: &str = include_str!("../config/t1024_rcw.xml");
const T1040_XML: &str = include_str!("../config/t1040_rcw.xml");
const T2080_XML: &str = include_str!("../config/t2080_rcw.xml");
const T4240_XML: &str = include_str!("../config/t4240_rcw.xml");

fn get_xml_for_processor(processor: &str) -> Option<&'static str> {
    match processor {
        "P2041" => Some(P2041_XML),
        "P3041" => Some(P3041_XML),
        "P4080" => Some(P4080_XML),
        // P5010 is a single-core P5020 variant with identical RCW layout
        "P5020" | "P5010" => Some(P5020_XML),
        "P5040" => Some(P5040_XML),
        // T1023 is a single-core T1024 variant with identical RCW layout
        "T1024" | "T1023" => Some(T1024_XML),
        // T1042 is a T1040 variant with different DDR3L support; identical RCW layout
        "T1040" | "T1042" => Some(T1040_XML),
        // T2081 is a T2080 variant (4 cores vs 8) with identical RCW layout
        "T2080" | "T2081" => Some(T2080_XML),
        // T4160 is a 6-core T4240 variant with identical RCW layout
        "T4240" | "T4160" => Some(T4240_XML),
        _ => None,
    }
}

#[derive(Serialize)]
pub struct ParsedField {
    pub name: String,
    pub description: String,
    pub bit_offset: usize,
    pub width: usize,
    pub raw_value: u64,
    pub raw_hex: String,
    /// Some(meaning) if a match was found in the XML, None otherwise
    pub meaning: Option<String>,
}

#[derive(Serialize)]
pub struct ParseResult {
    pub processor: String,
    pub fields: Vec<ParsedField>,
    /// None = no end command found in data; Some(true/false) = CRC present and result
    pub crc_ok: Option<bool>,
    pub crc_stored: Option<String>,
    pub crc_computed: Option<String>,
}

/// Returns the crate version string from Cargo.toml.
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns a JSON array of available processor names.
#[wasm_bindgen]
pub fn list_processors() -> String {
    serde_json::to_string(&[
        "P2041", "P3041", "P4080", "P5020", "P5040",
        "T1024", "T1040", "T2080", "T4240",
    ]).unwrap()
}

/// Parses a PBL binary for the given processor.
/// Returns a JSON string of ParseResult on success, or an error string prefixed with "ERROR:".
#[wasm_bindgen]
pub fn parse_rcw(data: &[u8], processor: &str) -> String {
    console_error_panic_hook::set_once();

    let xml = match get_xml_for_processor(processor) {
        Some(x) => x,
        None => return format!("ERROR: Unknown processor '{processor}'"),
    };

    let cfg = match config::parse_config(xml) {
        Ok(c) => c,
        Err(e) => return format!("ERROR: Config parse failed: {e}"),
    };

    let rcw_bytes = match pbl::extract_rcw(data) {
        Ok(b) => b,
        Err(e) => return format!("ERROR: {e}"),
    };

    let mut fields: Vec<ParsedField> = Vec::new();

    for field_def in &cfg.fields {
        let raw = rcw::extract_bits(&rcw_bytes, field_def.bit_offset, field_def.width);
        let meaning = field_def.values.get(&raw).cloned();

        fields.push(ParsedField {
            name: field_def.name.clone(),
            description: field_def.description.clone(),
            bit_offset: field_def.bit_offset,
            width: field_def.width,
            raw_value: raw,
            raw_hex: format!("0x{:X}", raw),
            meaning,
        });
    }

    let (crc_ok, crc_stored, crc_computed) = match pbl::check_pbl_crc(data) {
        Some(r) => (
            Some(r.ok),
            Some(format!("0x{:08X}", r.stored)),
            Some(format!("0x{:08X}", r.computed)),
        ),
        None => (None, None, None),
    };

    let result = ParseResult {
        processor: cfg.processor,
        fields,
        crc_ok,
        crc_stored,
        crc_computed,
    };

    match serde_json::to_string(&result) {
        Ok(json) => json,
        Err(e) => format!("ERROR: JSON serialization failed: {e}"),
    }
}
