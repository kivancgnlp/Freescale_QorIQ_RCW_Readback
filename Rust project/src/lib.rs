mod config;
mod pbl;
mod rcw;

use serde::Serialize;
use wasm_bindgen::prelude::*;

// Embed processor configs at compile time
const P3041_XML: &str = include_str!("../config/p3041_rcw.xml");
const T2080_XML: &str = include_str!("../config/t2080_rcw.xml");

fn get_xml_for_processor(processor: &str) -> Option<&'static str> {
    match processor {
        "P3041" => Some(P3041_XML),
        // T2081 is a T2080 variant (4 cores vs 8) with identical RCW layout
        "T2080" | "T2081" => Some(T2080_XML),
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
}

/// Returns a JSON array of available processor names.
#[wasm_bindgen]
pub fn list_processors() -> String {
    serde_json::to_string(&["P3041", "T2080", "T2081"]).unwrap()
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

    let result = ParseResult {
        processor: cfg.processor,
        fields,
    };

    match serde_json::to_string(&result) {
        Ok(json) => json,
        Err(e) => format!("ERROR: JSON serialization failed: {e}"),
    }
}
