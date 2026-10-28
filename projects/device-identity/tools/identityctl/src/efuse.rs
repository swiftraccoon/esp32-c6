//! Reduces a private, filtered espefuse 4.12.0 JSON capture to fixed metadata.
//!
//! No raw values or input-provided strings are retained in the report. This
//! validates a saved representation; it does not authenticate hardware, detect
//! all eFuse coding errors, or establish uniqueness or application use.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Report {
    schema_version: u32,
    kind: ReportKind,
    observer: Observer,
    fields: [FieldSummary; 3],
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReportKind {
    EfuseSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Observer {
    PhysicalDebug,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct FieldSummary {
    field: Field,
    bits: u32,
    readable: bool,
    writeable: bool,
    content: Content,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Field {
    FactoryBaseMac,
    MacExtension,
    OptionalUniqueId,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Content {
    AllZero,
    Nonzero,
    Unknown,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    #[serde(rename = "MAC")]
    mac: InputField,
    #[serde(rename = "MAC_EXT")]
    mac_ext: InputField,
    #[serde(rename = "OPTIONAL_UNIQUE_ID")]
    optional_unique_id: InputField,
}

// Deliberately lacks Debug: even parse diagnostics must not include private data.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputField {
    name: String,
    value: String,
    raw_value: String,
    readable: bool,
    writeable: bool,
    #[serde(rename = "description")]
    _description: String,
    #[serde(rename = "category")]
    _category: String,
    block: u32,
    word: u32,
    pos: u32,
    efuse_type: String,
    bit_len: u32,
}

/// Validates the exact three-field capture and returns a nonidentifying summary.
///
/// Diagnostics are static strings. Protected fields remain unknown even when
/// their raw representation contains zeros. A nonzero optional ID establishes
/// only nonzero readable bits, with no claim about uniqueness or provisioning.
pub(crate) fn summarize(bytes: &[u8]) -> Result<Report, &'static str> {
    let input: Input =
        serde_json::from_slice(bytes).map_err(|_| "invalid filtered espefuse JSON schema")?;
    Ok(Report {
        schema_version: 1,
        kind: ReportKind::EfuseSummary,
        observer: Observer::PhysicalDebug,
        fields: [
            summarize_field(input.mac, Field::FactoryBaseMac)?,
            summarize_field(input.mac_ext, Field::MacExtension)?,
            summarize_field(input.optional_unique_id, Field::OptionalUniqueId)?,
        ],
    })
}

fn summarize_field(input: InputField, field: Field) -> Result<FieldSummary, &'static str> {
    let (name, byte_count, efuse_type, block, word, pos) = match field {
        Field::FactoryBaseMac => ("MAC", 6, "bytes:6", 1, 0, 0),
        Field::MacExtension => ("MAC_EXT", 2, "bytes:2", 1, 1, 16),
        Field::OptionalUniqueId => ("OPTIONAL_UNIQUE_ID", 16, "bytes:16", 2, 0, 0),
    };
    if input.name != name
        || input.bit_len != byte_count * 8
        || input.efuse_type != efuse_type
        || (input.block, input.word, input.pos) != (block, word, pos)
    {
        return Err("unexpected filtered eFuse field metadata");
    }

    let raw = input
        .raw_value
        .strip_prefix("0x")
        .filter(|raw| raw.len() == byte_count as usize * 2)
        .ok_or("invalid eFuse raw value shape")?;
    let raw_bytes = decode_hex(raw)?;
    let is_mac_field = matches!(field, Field::FactoryBaseMac | Field::MacExtension);
    let (value, separator) = if is_mac_field {
        (
            input
                .value
                .strip_suffix(" (OK)")
                .ok_or("invalid or unsuccessful eFuse MAC integrity annotation")?,
            ':',
        )
    } else {
        (input.value.as_str(), ' ')
    };
    let tokens: Vec<_> = value.split(separator).collect();
    if tokens.len() != byte_count as usize
        || tokens.iter().any(|token| {
            token.len() != 2
                || !token
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() || (!input.readable && byte == b'?'))
        })
    {
        return Err("invalid eFuse formatted value shape");
    }

    let content = if input.readable {
        let mut formatted = decode_hex(&tokens.concat())?;
        // The pinned MAC subclass formats get_raw() directly. The generic
        // bytes formatter and JSON raw_value formatter both reverse bits.bytes.
        // MAC and MAC_EXT both use that subclass, selected by the pinned
        // field-definition loader. Both have opposite byte order in value
        // versus raw_value; OPTIONAL_UNIQUE_ID uses the generic formatter.
        if is_mac_field {
            formatted.reverse();
        }
        if formatted != raw_bytes {
            return Err("inconsistent eFuse formatted and raw values");
        }
        if raw_bytes.iter().all(|&byte| byte == 0) {
            Content::AllZero
        } else {
            Content::Nonzero
        }
    } else {
        Content::Unknown
    };

    Ok(FieldSummary {
        field,
        bits: byte_count * 8,
        readable: input.readable,
        writeable: input.writeable,
        content,
    })
}

fn decode_hex(value: &str) -> Result<Vec<u8>, &'static str> {
    if !value.len().is_multiple_of(2)
        || !value.is_ascii()
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("invalid hexadecimal eFuse value");
    }
    (0..value.len())
        .step_by(2)
        .map(|offset| {
            // Even length and ASCII establish both slice boundaries.
            u8::from_str_radix(&value[offset..offset + 2], 16)
                .map_err(|_| "invalid hexadecimal eFuse value")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{Content, summarize};

    fn fixture(all_zero: bool) -> Value {
        let mut root = serde_json::Map::new();
        for (name, count, block, word, pos) in [
            ("MAC", 6, 1, 0, 0),
            ("MAC_EXT", 2, 1, 1, 16),
            ("OPTIONAL_UNIQUE_ID", 16, 2, 0, 0),
        ] {
            let raw: Vec<_> = (0..count)
                .map(|index| if all_zero { 0 } else { index + 1 })
                .collect();
            let mut formatted = raw.clone();
            let is_mac_field = name == "MAC" || name == "MAC_EXT";
            if is_mac_field {
                formatted.reverse();
            }
            let separator = if is_mac_field { ":" } else { " " };
            let mut value = formatted
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<Vec<_>>()
                .join(separator);
            if is_mac_field {
                value.push_str(" (OK)");
            }
            root.insert(
                name.into(),
                json!({
                    "name": name,
                    "value": value,
                    "raw_value": format!("0x{}", raw.iter().map(|byte| format!("{byte:02x}")).collect::<String>()),
                    "readable": true,
                    "writeable": false,
                    "description": "synthetic description",
                    "category": "synthetic category",
                    "block": block,
                    "word": word,
                    "pos": pos,
                    "efuse_type": format!("bytes:{count}"),
                    "bit_len": count * 8,
                }),
            );
        }
        Value::Object(root)
    }

    fn parse(value: &Value) -> Result<super::Report, &'static str> {
        summarize(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn accepts_zero_and_nonuniform_byte_order_controls() {
        for (zero, expected) in [(true, Content::AllZero), (false, Content::Nonzero)] {
            let report = parse(&fixture(zero)).unwrap();
            assert!(report.fields.iter().all(|field| field.content == expected));
            let encoded = serde_json::to_vec(&report).unwrap();
            let decoded: super::Report = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(decoded.fields.len(), 3);
        }
    }

    #[test]
    fn protected_zero_fields_remain_unknown() {
        let mut input = fixture(true);
        for field in input.as_object_mut().unwrap().values_mut() {
            field["readable"] = json!(false);
            field["value"] = json!(field["value"].as_str().unwrap().replace('0', "?"));
        }
        let report = parse(&input).unwrap();
        assert!(
            report
                .fields
                .iter()
                .all(|field| field.content == Content::Unknown)
        );
    }

    #[test]
    fn mac_extension_requires_sdk_mac_format_integrity_and_byte_order() {
        // The actual SDK virtual capture has a two-byte colon-formatted MAC
        // extension and the same integrity suffix as MAC. A generic bytes
        // fixture previously hid this distinction. Unequal synthetic bytes
        // additionally detect a reversal mistake that zero bytes cannot.
        let good = fixture(false);
        assert!(parse(&good).is_ok());
        let raw = good["MAC_EXT"]["raw_value"]
            .as_str()
            .unwrap()
            .strip_prefix("0x")
            .unwrap();
        let tokens: Vec<_> = raw.as_bytes().chunks_exact(2).collect();
        let ascending = tokens
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).unwrap())
            .collect::<Vec<_>>();
        for wrong_value in [
            ascending.join(" "),
            format!("{} (OK)", ascending.join(":")),
            good["MAC_EXT"]["value"]
                .as_str()
                .unwrap()
                .replace(" (OK)", " (Block1 has ERRORS:1 FAIL:1)"),
        ] {
            let mut input = good.clone();
            input["MAC_EXT"]["value"] = json!(wrong_value);
            assert!(parse(&input).is_err());
        }
    }

    #[test]
    fn rejects_missing_extra_duplicate_and_malformed_inputs() {
        let good = fixture(false);
        let mut missing = good.clone();
        missing.as_object_mut().unwrap().remove("MAC_EXT");
        assert!(parse(&missing).is_err());
        let mut extra = good.clone();
        extra["KEY_BLOCK"] = json!({});
        assert!(parse(&extra).is_err());
        let mut duplicate = serde_json::to_string(&good).unwrap();
        duplicate.pop();
        duplicate.push_str(&format!(",\"MAC\":{}}}", good["MAC"]));
        assert!(summarize(duplicate.as_bytes()).is_err());
        assert!(summarize(b"not JSON").is_err());
        assert!(summarize(b"{}").is_err());
        assert!(summarize(&[255]).is_err());
    }

    #[test]
    fn rejects_inconsistent_metadata_values_and_integrity_failures() {
        for (field, key, value) in [
            ("MAC", "name", json!("unexpected")),
            ("MAC", "bit_len", json!(47)),
            ("MAC_EXT", "pos", json!(0)),
            ("OPTIONAL_UNIQUE_ID", "efuse_type", json!("bytes:15")),
            ("OPTIONAL_UNIQUE_ID", "readable", json!("true")),
            ("MAC_EXT", "raw_value", json!("0x00")),
            ("MAC_EXT", "value", json!("garbage")),
            (
                "MAC",
                "value",
                json!("synthetic (Block1 has ERRORS:1 FAIL:1)"),
            ),
        ] {
            let mut input = fixture(false);
            input[field][key] = value;
            assert!(parse(&input).is_err(), "{field}/{key}");
        }
        let mut input = fixture(false);
        input["MAC"]["raw_value"] = fixture(true)["MAC"]["raw_value"].clone();
        assert!(parse(&input).is_err());
    }

    #[test]
    fn report_contains_no_input_strings_or_raw_values() {
        let mut input = fixture(false);
        let marker = ["private", "marker", "never", "publish"].join("-");
        for field in input.as_object_mut().unwrap().values_mut() {
            field["description"] = json!(marker);
            field["category"] = json!(marker);
        }
        let encoded = serde_json::to_string(&parse(&input).unwrap()).unwrap();
        assert!(!encoded.contains(&marker));
        for field in input.as_object().unwrap().values() {
            assert!(!encoded.contains(field["value"].as_str().unwrap()));
            assert!(!encoded.contains(field["raw_value"].as_str().unwrap()));
        }
    }
}
