use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub bit_offset: usize,
    pub width: usize,
    pub description: String,
    /// Maps raw integer value → human-readable meaning
    pub values: HashMap<u64, String>,
}

#[derive(Debug)]
pub struct RcwConfig {
    pub processor: String,
    pub rcw_bits: usize,
    pub fields: Vec<FieldDef>,
}

pub fn parse_config(xml: &str) -> Result<RcwConfig, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut processor = String::new();
    let mut rcw_bits: usize = 512;
    let mut fields: Vec<FieldDef> = Vec::new();
    let mut current_field: Option<FieldDef> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"rcw_config" => {
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"processor" => {
                                processor = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                            b"rcw_bits" => {
                                rcw_bits = String::from_utf8_lossy(&attr.value)
                                    .parse()
                                    .map_err(|_| "Invalid rcw_bits")?;
                            }
                            _ => {}
                        }
                    }
                }
                b"field" => {
                    current_field = Some(parse_field_attrs(e)?);
                }
                _ => {}
            },

            Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"field" => {
                    // Self-closing <field .../> — no child <value> elements
                    fields.push(parse_field_attrs(e)?);
                }
                b"value" => {
                    if let Some(ref mut field) = current_field {
                        if let Some((enc, meaning)) = parse_value_attrs(e) {
                            field.values.insert(enc, meaning);
                        }
                    }
                }
                _ => {}
            },

            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"field" {
                    if let Some(field) = current_field.take() {
                        fields.push(field);
                    }
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
    }

    Ok(RcwConfig {
        processor,
        rcw_bits,
        fields,
    })
}

fn parse_field_attrs(e: &quick_xml::events::BytesStart) -> Result<FieldDef, String> {
    let mut name = String::new();
    let mut bit_offset: usize = 0;
    let mut width: usize = 1;
    let mut description = String::new();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name" => name = String::from_utf8_lossy(&attr.value).into_owned(),
            b"bit_offset" => {
                bit_offset = String::from_utf8_lossy(&attr.value)
                    .parse()
                    .map_err(|_| format!("Invalid bit_offset for field '{name}'"))?;
            }
            b"width" => {
                width = String::from_utf8_lossy(&attr.value)
                    .parse()
                    .map_err(|_| format!("Invalid width for field '{name}'"))?;
            }
            b"description" => description = String::from_utf8_lossy(&attr.value).into_owned(),
            _ => {}
        }
    }

    Ok(FieldDef {
        name,
        bit_offset,
        width,
        description,
        values: HashMap::new(),
    })
}

fn parse_value_attrs(e: &quick_xml::events::BytesStart) -> Option<(u64, String)> {
    let mut encoding: Option<u64> = None;
    let mut meaning = String::new();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"encoding" => {
                encoding = String::from_utf8_lossy(&attr.value).parse::<u64>().ok();
            }
            b"meaning" => {
                meaning = String::from_utf8_lossy(&attr.value).into_owned();
            }
            _ => {}
        }
    }

    encoding.map(|enc| (enc, meaning))
}
