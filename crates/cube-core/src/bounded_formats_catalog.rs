//! Versioned contracts for bounded lossless coding and structural inspection.
use crate::{
    binary::compression::{bzip2, dictionary, huffman, lz4, lzw, probability},
    errors::{ErrorCode, OperationError},
    forensics::formats::{bmp, executable, mpeg, ogg, tiff, webp},
    operations::{self as o, contracts as c, metadata as m},
};
use schemars::{JsonSchema, Schema};
use serde::Deserialize;
use serde_json::{Value, json};

const KIB64: usize = 65_536;
const MIB: usize = 1 << 20;
const RAW: usize = 16 * MIB;
const STRUCTURE: usize = 32 * MIB;
pub const MAX_DOCUMENT: usize = 48 * MIB;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoOptions {}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    #[schemars(range(max = 16_777_216))]
    pub expected_length: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Window {
    #[serde(default = "default_window")]
    #[schemars(range(min = 1, max = 4096))]
    pub window: u16,
}
fn default_window() -> u16 {
    dictionary::DEFAULT_WINDOW
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Permutation {
    #[schemars(length(min = 256, max = 256))]
    pub alphabet: Option<Vec<u8>>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Primary {
    #[schemars(range(max = 65_535))]
    pub primary_index: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Lz4Encode {
    pub mode: lz4::Mode,
    #[serde(default)]
    pub limits: lz4::Limits,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Lz4Decode {
    Block {
        #[schemars(range(max = 16_777_216))]
        expected_length: usize,
        #[serde(default)]
        limits: lz4::Limits,
    },
    Frame {
        #[serde(default)]
        limits: lz4::Limits,
    },
}
impl Lz4Decode {
    pub fn parts(self) -> (lz4::Decode, lz4::Limits) {
        match self {
            Self::Block {
                expected_length,
                limits,
            } => (lz4::Decode::Block { expected_length }, limits),
            Self::Frame { limits } => (lz4::Decode::Frame, limits),
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bzip2Level {
    #[serde(default = "default_level")]
    #[schemars(range(min = 1, max = 9))]
    pub level: u8,
}
fn default_level() -> u8 {
    bzip2::DEFAULT_LEVEL
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bzip2Members {
    #[serde(default)]
    pub concatenate: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HuffmanCounts {
    #[schemars(range(max = 16_777_216))]
    pub bit_count: usize,
    #[schemars(range(max = 8_388_608))]
    pub symbol_count: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frequency {
    pub symbol: u8,
    #[schemars(length(min = 1, max = 20), regex(pattern = "^(0|[1-9][0-9]{0,19})$"))]
    pub count: String,
}

/// Convert the exact wire table to the Core's fixed indexed model.
pub fn frequencies(records: Vec<Frequency>) -> Result<[u64; 256], OperationError> {
    if records.len() > 256 {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "frequency table exceeds 256 symbols",
        ));
    }
    let invalid = || {
        OperationError::new(
            ErrorCode::InvalidInput,
            "use unique symbols and canonical u64 decimal counts",
        )
    };
    let mut seen = [false; 256];
    let mut result = [0; 256];
    for Frequency { symbol, count } in records {
        let i = symbol as usize;
        if std::mem::replace(&mut seen[i], true) {
            return Err(invalid());
        }
        let value: u64 = count.parse().map_err(|_| invalid())?;
        if value.to_string() != count {
            return Err(invalid());
        }
        result[i] = value;
    }
    Ok(result)
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntropyMode {
    pub profile: probability::Profile,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntropyDecode {
    pub profile: probability::Profile,
    #[schemars(range(max = 65_536))]
    pub symbol_count: usize,
}

pub const PACKBITS_ENCODE: &str = "binary.packbits.encode";
pub const PACKBITS_DECODE: &str = "binary.packbits.decode";
const EXPECTED_PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "expected_length",
    description: "Optional exact decoded byte count.",
    default: None,
    required: false,
    constraints: o::ParameterType::Unsigned {
        min: 0,
        max: RAW as u64,
    },
}];

const fn bytes<const N: usize>(name: &'static str) -> c::Port {
    c::Port {
        name,
        semantic_type: "bytes",
        required: true,
        schema: c::bytes_schema::<N>,
    }
}
const fn typed<T: JsonSchema>(name: &'static str) -> c::Port {
    c::Port {
        name,
        semantic_type: "json",
        required: true,
        schema: c::schema::<T>,
    }
}
fn array<T: JsonSchema, const N: usize>() -> Schema {
    let mut schema = c::schema::<Vec<T>>().to_value();
    schema["maxItems"] = json!(N);
    Schema::try_from(schema).expect("bounded list")
}
const fn list<T: JsonSchema, const N: usize>(name: &'static str) -> c::Port {
    c::Port {
        name,
        semantic_type: "json",
        required: true,
        schema: array::<T, N>,
    }
}

const fn limits(
    input: usize,
    output: usize,
    items: Option<u64>,
    work: Option<u64>,
    scratch: Option<u64>,
) -> c::Limits {
    c::Limits {
        max_input_bytes: Some(input as u64),
        max_output_bytes: Some(output as u64),
        max_items: items,
        max_work: work,
        max_memory_bytes: scratch,
    }
}

const WORK: Option<u64> = Some(16 << 20);
const SMALL: Option<u64> = Some(8 << 20);
const LARGE: Option<u64> = Some(64 << 20);
const SOURCE: c::Port = bytes::<STRUCTURE>("data");
const DATA: c::Port = bytes::<MIB>("data");
const PACKBITS_INPUT: c::Port = bytes::<MIB>("input");
const BLOCK: c::Port = bytes::<KIB64>("data");
const RAW_DATA: c::Port = bytes::<RAW>("data");
const RAW_OUT: c::Port = bytes::<RAW>("output");
const MIB_OUT: c::Port = bytes::<MIB>("output");
const BLOCK_OUT: c::Port = bytes::<KIB64>("output");
const LZ77: c::Port = list::<dictionary::Lz77Token, KIB64>("tokens");
const LZ78: c::Port = list::<dictionary::Lz78Token, KIB64>("tokens");
const CODES: c::Port = list::<huffman::Code, 256>("codes");
const FREQUENCIES: c::Port = list::<Frequency, 256>("frequencies");
const LENGTHS: c::Port = list::<huffman::Length, 256>("lengths");
const PACKED: c::Port = bytes::<{ 8 * MIB }>("output");
const WIRE: c::Port = bytes::<{ probability::MAX_WIRE_BYTES }>("data");
const WIRE_OUT: c::Port = bytes::<{ probability::MAX_WIRE_BYTES }>("output");

// Keep union validation exact while listing the fields in the parameter model
// consumed by the CLI/discovery and graph form renderers.
fn profile_fields<T: JsonSchema>() -> Schema {
    let mut value = c::schema::<T>().to_value();
    let variants = value["oneOf"].as_array().expect("profile union");
    let mut properties = serde_json::Map::new();
    let mut choices = std::collections::BTreeMap::<String, Vec<Value>>::new();
    let mut required = std::collections::BTreeSet::<String>::new();
    for (index, variant) in variants.iter().enumerate() {
        let fields = variant["properties"].as_object().expect("profile fields");
        for (name, field) in fields {
            if let Some(constant) = field.get("const") {
                choices
                    .entry(name.clone())
                    .or_default()
                    .push(constant.clone());
            } else {
                properties
                    .entry(name.clone())
                    .or_insert_with(|| field.clone());
            }
        }
        let names = variant["required"].as_array().expect("profile selector");
        if index == 0 {
            required.extend(names.iter().filter_map(Value::as_str).map(str::to_owned));
        } else {
            required.retain(|name| names.iter().any(|field| field.as_str() == Some(name)));
        }
    }
    for (name, variants) in choices {
        properties.insert(name, json!({"type":"string","enum":variants}));
    }
    value["type"] = json!("object");
    value["properties"] = Value::Object(properties);
    value["required"] = json!(required);
    Schema::try_from(value).expect("tagged profile schema")
}

const fn profile_row<T: JsonSchema>(mut value: Row) -> Row {
    value.contract.parameters = profile_fields::<T>;
    value
}

struct Row {
    metadata: m::Metadata,
    contract: c::Entry,
}

macro_rules! row {
    ($id:expr, $slug:literal, $category:literal, $name:literal, $family:literal,
     $description:literal, $old:literal => $new:literal, $parameters:ty, $legacy:expr,
     $input:expr => $output:expr, $limits:expr, $reverse:expr, $rust:literal, $py:literal,
     $reference:literal, $sample:literal => $answer:literal) => {
        Row {
            metadata: m::Metadata {
                schema_version: 1, canonical_name: $name, family: $family, aliases: &[],
                security: m::Security {
                    classification: m::Classification::BinaryTransform,
                    limitations: $description,
                    private_input: false, private_output: false, sensitive_parameters: &[],
                },
                profile: m::Profile { variant: $description, alphabet: "Exact bytes and typed records.",
                    key: "None.", key_advancement: "Not applicable.", block_bits: None, output_bits: None },
                resources: &[], resource_notes: "The versioned contract lists the Core ceilings. Hosts may impose smaller limits.",
                rust_api: concat!("cube_core::", $rust), python_api: $py, references: &[$reference],
            },
            contract: c::Entry {
                operation_version: 1,
                descriptor: o::Descriptor {
                    id: $id, category: $category, description: $description,
                    input_type: $old, output_type: $new, parameters: $legacy,
                    deterministic: true, reversible: $reverse,
                    command: concat!("binary format ", $slug),
                    example: concat!("cube binary format ", $slug, " --text '", $sample, "'"),
                },
                inputs: $input, outputs: $output, parameters: c::schema::<$parameters>,
                limits: $limits,
                fixture: || {
                    let mut data: Value = serde_json::from_str($sample).expect("bounded format fixture");
                    Ok(c::Fixture { inputs: data["inputs"].take(), parameters: data["parameters"].take(),
                        outputs: serde_json::from_str($answer).expect("bounded format known answer") })
                },
            },
        }
    };
}

const ROWS: [Row; 27] = [
    row!(PACKBITS_ENCODE, "packbits-encode", "binary", "Encode PackBits", "bounded compression",
        "Compress bytes using TIFF 6.0 PackBits run and literal controls.", "bytes" => "bytes", NoOptions, &[],
        &[PACKBITS_INPUT] => &[RAW_OUT], limits(MIB, RAW, None, WORK, SMALL), Some(true),
        "binary::compression::packbits::encode", "packbits_encode", "https://pdfa.org/wp-content/uploads/2020/07/TIFF6.pdf",
        r#"{"inputs":{"input":{"base64":"UVFRWFhZWlpaWg=="}},"parameters":{}}"# =>
        r#"{"output":{"base64":"/lECWFhZ/Vo="}}"#),
    row!("binary.lz77.encode", "lz77-encode", "binary", "Encode LZ77 tokens", "bounded compression",
        "Construct deterministic explicit LZ77 dictionary tokens.", "bytes" => "json", Window, &[],
        &[BLOCK] => &[LZ77], limits(KIB64, 4 * MIB, Some(65_536), WORK, SMALL), Some(true),
        "binary::compression::dictionary::lz77_encode", "lz77_encode", "https://doi.org/10.1109/TIT.1977.1055714",
        r#"{"inputs":{"data":{"base64":"QUFBQUFB"}},"parameters":{}}"# =>
        r#"{"tokens":[{"kind":"literal","byte":65},{"kind":"match","distance":1,"length":5}]}"#),
    row!("binary.lz78.encode", "lz78-encode", "binary", "Encode LZ78 tokens", "bounded compression",
        "Construct deterministic explicit LZ78 dictionary tokens.", "bytes" => "json", NoOptions, &[],
        &[BLOCK] => &[LZ78], limits(KIB64, 4 * MIB, Some(65_536), WORK, SMALL), Some(true),
        "binary::compression::dictionary::lz78_encode", "lz78_encode", "https://doi.org/10.1109/TIT.1978.1055934",
        r#"{"inputs":{"data":{"base64":"QUFBQUFBQQ=="}},"parameters":{}}"# =>
        r#"{"tokens":[{"index":0,"byte":65},{"index":1,"byte":65},{"index":2,"byte":65},{"index":1,"byte":null}]}"#),
    row!("binary.lz4.encode", "lz4-encode", "binary", "Encode LZ4", "bounded compression",
        "Encode a standard LZ4 frame or an explicitly selected raw block.", "bytes" => "bytes", Lz4Encode, &[],
        &[RAW_DATA] => &[RAW_OUT], limits(RAW, RAW, Some(10_000), None, LARGE), Some(true),
        "binary::compression::lz4::encode", "lz4_encode", "https://github.com/lz4/lz4/tree/v1.10.0/doc",
        r#"{"inputs":{"data":{"base64":"YWJj"}},"parameters":{"mode":"block"}}"# =>
        r#"{"output":{"base64":"MGFiYw=="}}"#),
    row!("binary.bzip2.encode", "bzip2-encode", "binary", "Encode bzip2", "bounded compression",
        "Encode bounded standard BZh streams with verified framing.", "bytes" => "bytes", Bzip2Level, &[],
        &[RAW_DATA] => &[RAW_OUT], limits(RAW, RAW, Some(10_000), None, LARGE), Some(true),
        "binary::compression::bzip2::encode", "bzip2_encode", "https://sourceware.org/bzip2/manual/manual.html",
        r#"{"inputs":{"data":{"base64":"YWJj"}},"parameters":{}}"# =>
        r#"{"output":{"base64":"QlpoMTFBWSZTWWSMu3MAAAABADgAIAAhmBmEYXckU4UJBkjLtzA="}}"#),
    profile_row::<lzw::Profile>(
        row!("binary.lzw.encode", "lzw-encode", "binary", "Encode GIF/TIFF LZW", "bounded compression",
        "Encode an explicit GIF89a or TIFF 6.0 LZW bitstream.", "bytes" => "bytes", lzw::Profile, &[],
        &[DATA] => &[RAW_OUT], limits(MIB, RAW, Some(4096), WORK, SMALL), Some(true),
        "binary::compression::lzw::encode", "lzw_encode", "https://www.w3.org/Graphics/GIF/spec-gif89a.txt",
        r#"{"inputs":{"data":{"base64":"AAEAAQ=="}},"parameters":{"profile":"gif","min_code_size":2}}"# =>
        r#"{"output":{"base64":"RFw="}}"#),
    ),
    row!("binary.bwt.encode", "bwt-encode", "binary", "Encode Burrows–Wheeler", "bounded compression",
        "Order cyclic rotations and return the last column and primary index.", "bytes" => "json", NoOptions, &[],
        &[BLOCK] => &[BLOCK_OUT, typed::<usize>("primary_index")], limits(KIB64, KIB64 + 32, None, WORK, SMALL), Some(true),
        "binary::compression::transforms::bwt", "bwt_encode", "https://www.hpl.hp.com/techreports/Compaq-DEC/SRC-RR-124.pdf",
        r#"{"inputs":{"data":{"base64":"QkFOQU5B"}},"parameters":{}}"# =>
        r#"{"output":{"base64":"Tk5CQUFB"},"primary_index":3}"#),
    row!("binary.mtf.encode", "mtf-encode", "binary", "Encode move-to-front", "bounded compression",
        "Apply the move-to-front byte transform with an optional exact alphabet.", "bytes" => "bytes", Permutation, &[],
        &[BLOCK] => &[BLOCK_OUT], limits(KIB64, KIB64, None, WORK, SMALL), Some(true),
        "binary::compression::transforms::mtf", "mtf_encode", "https://www.hpl.hp.com/techreports/Compaq-DEC/SRC-RR-124.pdf",
        r#"{"inputs":{"data":{"base64":"YmFuYW5h"}},"parameters":{}}"# =>
        r#"{"output":{"base64":"YmJuAQEB"}}"#),
    row!("binary.huffman.encode", "huffman-encode", "binary", "Encode canonical Huffman", "bounded compression",
        "Encode MSB-first canonical Huffman codes and return the exact bit count and code table.", "json" => "json", NoOptions, &[],
        &[DATA, FREQUENCIES] => &[PACKED, CODES, typed::<usize>("bit_count")], limits(MIB + 16_384, 8 * MIB, Some(256), WORK, SMALL), Some(true),
        "binary::compression::huffman::encode", "huffman_encode", "https://doi.org/10.1109/JRPROC.1952.273898",
        r#"{"inputs":{"frequencies":[{"symbol":0,"count":"7"},{"symbol":1,"count":"7"},{"symbol":2,"count":"7"},{"symbol":3,"count":"7"}],"data":{"base64":"AAECAwE="}},"parameters":{}}"# =>
        r#"{"codes":[{"symbol":0,"length":2,"code":0},{"symbol":1,"length":2,"code":1},{"symbol":2,"length":2,"code":2},{"symbol":3,"length":2,"code":3}],"output":{"base64":"G0A="},"bit_count":10}"#),
    row!("binary.entropy.encode", "entropy-encode", "binary", "Encode arithmetic or range stream", "bounded compression",
        "Encode a fixed model with explicitly selected integer arithmetic or carry-buffered range coding.", "json" => "bytes", EntropyMode, &[],
        &[BLOCK, FREQUENCIES] => &[WIRE_OUT], limits(probability::MAX_WIRE_BYTES + 16_384, probability::MAX_WIRE_BYTES, Some(256), WORK, SMALL), Some(true),
        "binary::compression::probability::encode", "entropy_encode", "https://doi.org/10.1145/214762.214771",
        r#"{"inputs":{"frequencies":[{"symbol":65,"count":"1"},{"symbol":66,"count":"1"}],"data":{"base64":"QUJCQQ=="}},"parameters":{"profile":"arithmetic16"}}"# =>
        r#"{"output":{"base64":"ZA=="}}"#),
    row!(PACKBITS_DECODE, "packbits-decode", "binary", "Decode PackBits", "bounded compression",
        "Decode checked TIFF 6.0 PackBits runs and optionally verify the expected byte count.", "bytes" => "bytes", Expected, EXPECTED_PARAMETERS,
        &[PACKBITS_INPUT] => &[RAW_OUT], limits(MIB, RAW, None, WORK, SMALL), Some(true),
        "binary::compression::packbits::decode", "packbits_decode", "https://pdfa.org/wp-content/uploads/2020/07/TIFF6.pdf",
        r#"{"inputs":{"input":{"base64":"/lECWFhZ/Vo="}},"parameters":{"expected_length":10}}"# =>
        r#"{"output":{"base64":"UVFRWFhZWlpaWg=="}}"#),
    row!("binary.lz77.decode", "lz77-decode", "binary", "Decode LZ77 tokens", "bounded compression",
        "Expand and check explicit LZ77 dictionary tokens.", "json" => "bytes", Window, &[],
        &[LZ77] => &[MIB_OUT], limits(4 * MIB, MIB, Some(65_536), WORK, SMALL), Some(true),
        "binary::compression::dictionary::lz77_decode", "lz77_decode", "https://doi.org/10.1109/TIT.1977.1055714",
        r#"{"inputs":{"tokens":[{"kind":"literal","byte":65},{"kind":"match","distance":1,"length":5}]},"parameters":{}}"# =>
        r#"{"output":{"base64":"QUFBQUFB"}}"#),
    row!("binary.lz78.decode", "lz78-decode", "binary", "Decode LZ78 tokens", "bounded compression",
        "Expand and check explicit LZ78 dictionary tokens.", "json" => "bytes", NoOptions, &[],
        &[LZ78] => &[MIB_OUT], limits(4 * MIB, MIB, Some(65_536), WORK, SMALL), Some(true),
        "binary::compression::dictionary::lz78_decode", "lz78_decode", "https://doi.org/10.1109/TIT.1978.1055934",
        r#"{"inputs":{"tokens":[{"index":0,"byte":65},{"index":1,"byte":65},{"index":2,"byte":65},{"index":1,"byte":null}]},"parameters":{}}"# =>
        r#"{"output":{"base64":"QUFBQUFBQQ=="}}"#),
    profile_row::<Lz4Decode>(
        row!("binary.lz4.decode", "lz4-decode", "binary", "Decode LZ4", "bounded compression",
        "Decode a standard LZ4 frame or an explicitly selected raw block.", "bytes" => "bytes", Lz4Decode, &[],
        &[RAW_DATA] => &[RAW_OUT], limits(RAW, RAW, Some(10_000), None, LARGE), Some(true),
        "binary::compression::lz4::decode", "lz4_decode", "https://github.com/lz4/lz4/tree/v1.10.0/doc",
        r#"{"inputs":{"data":{"base64":"MGFiYw=="}},"parameters":{"mode":"block","expected_length":3}}"# =>
        r#"{"output":{"base64":"YWJj"}}"#),
    ),
    row!("binary.bzip2.decode", "bzip2-decode", "binary", "Decode bzip2", "bounded compression",
        "Decode bounded standard BZh streams with verified framing.", "bytes" => "bytes", Bzip2Members, &[],
        &[RAW_DATA] => &[RAW_OUT], limits(RAW, RAW, Some(10_000), None, LARGE), Some(true),
        "binary::compression::bzip2::decode", "bzip2_decode", "https://sourceware.org/bzip2/manual/manual.html",
        r#"{"inputs":{"data":{"base64":"QlpoMTFBWSZTWWSMu3MAAAABADgAIAAhmBmEYXckU4UJBkjLtzA="}},"parameters":{}}"# =>
        r#"{"output":{"base64":"YWJj"}}"#),
    profile_row::<lzw::Profile>(
        row!("binary.lzw.decode", "lzw-decode", "binary", "Decode GIF/TIFF LZW", "bounded compression",
        "Decode an explicit GIF89a or TIFF 6.0 LZW bitstream.", "bytes" => "bytes", lzw::Profile, &[],
        &[DATA] => &[RAW_OUT], limits(MIB, RAW, Some(4096), WORK, SMALL), Some(true),
        "binary::compression::lzw::decode", "lzw_decode", "https://www.w3.org/Graphics/GIF/spec-gif89a.txt",
        r#"{"inputs":{"data":{"base64":"RFw="}},"parameters":{"profile":"gif","min_code_size":2}}"# =>
        r#"{"output":{"base64":"AAEAAQ=="}}"#),
    ),
    row!("binary.bwt.decode", "bwt-decode", "binary", "Decode Burrows–Wheeler", "bounded compression",
        "Invert the last column using its explicit primary index.", "bytes" => "bytes", Primary, &[],
        &[BLOCK] => &[BLOCK_OUT], limits(KIB64, KIB64 + 32, None, WORK, SMALL), Some(true),
        "binary::compression::transforms::inverse_bwt", "bwt_decode", "https://www.hpl.hp.com/techreports/Compaq-DEC/SRC-RR-124.pdf",
        r#"{"inputs":{"data":{"base64":"Tk5CQUFB"}},"parameters":{"primary_index":3}}"# =>
        r#"{"output":{"base64":"QkFOQU5B"}}"#),
    row!("binary.mtf.decode", "mtf-decode", "binary", "Decode move-to-front", "bounded compression",
        "Invert byte indexes using the same initial move-to-front alphabet.", "bytes" => "bytes", Permutation, &[],
        &[BLOCK] => &[BLOCK_OUT], limits(KIB64, KIB64, None, WORK, SMALL), Some(true),
        "binary::compression::transforms::mtf", "mtf_decode", "https://www.hpl.hp.com/techreports/Compaq-DEC/SRC-RR-124.pdf",
        r#"{"inputs":{"data":{"base64":"YmJuAQEB"}},"parameters":{}}"# =>
        r#"{"output":{"base64":"YmFuYW5h"}}"#),
    row!("binary.huffman.decode", "huffman-decode", "binary", "Decode canonical Huffman", "bounded compression",
        "Decode an exact symbol and bit count with a checked canonical Huffman length table.", "json" => "bytes", HuffmanCounts, &[],
        &[DATA, LENGTHS] => &[PACKED], limits(MIB + 16_384, 8 * MIB, Some(256), WORK, SMALL), Some(true),
        "binary::compression::huffman::decode", "huffman_decode", "https://doi.org/10.1109/JRPROC.1952.273898",
        r#"{"inputs":{"lengths":[{"symbol":0,"length":2},{"symbol":1,"length":2},{"symbol":2,"length":2},{"symbol":3,"length":2}],"data":{"base64":"G0A="}},"parameters":{"symbol_count":5,"bit_count":10}}"# =>
        r#"{"output":{"base64":"AAECAwE="}}"#),
    row!("binary.entropy.decode", "entropy-decode", "binary", "Decode arithmetic or range stream", "bounded compression",
        "Decode the declared count and verify canonical arithmetic or range termination.", "json" => "bytes", EntropyDecode, &[],
        &[WIRE, FREQUENCIES] => &[BLOCK_OUT], limits(probability::MAX_WIRE_BYTES + 16_384, probability::MAX_WIRE_BYTES, Some(256), WORK, SMALL), Some(true),
        "binary::compression::probability::decode", "entropy_decode", "https://doi.org/10.1145/214762.214771",
        r#"{"inputs":{"frequencies":[{"symbol":65,"count":"1"},{"symbol":66,"count":"1"}],"data":{"base64":"AF////4="}},"parameters":{"profile":"range32","symbol_count":4}}"# =>
        r#"{"output":{"base64":"QUJCQQ=="}}"#),
    row!("binary.huffman.construct", "huffman-construct", "binary", "Construct canonical Huffman codes", "bounded compression",
        "Construct deterministic canonical byte codes from an exact frequency table.", "json" => "json", NoOptions, &[],
        &[FREQUENCIES] => &[CODES], limits(16_384, 16_384, Some(256), WORK, SMALL), None,
        "binary::compression::huffman::construct", "huffman_construct", "https://doi.org/10.1109/JRPROC.1952.273898",
        r#"{"inputs":{"frequencies":[{"symbol":0,"count":"7"},{"symbol":1,"count":"7"},{"symbol":2,"count":"7"},{"symbol":3,"count":"7"}]},"parameters":{}}"# =>
        r#"{"codes":[{"symbol":0,"length":2,"code":0},{"symbol":1,"length":2,"code":1},{"symbol":2,"length":2,"code":2},{"symbol":3,"length":2,"code":3}]}"#),
    row!("file.bmp.inspect", "bmp-inspect", "file", "Inspect BMP", "file framing",
        "Inspect a 40-byte Windows BMP header, dimensions and bounded raster extents.", "bytes" => "json", NoOptions, &[],
        &[SOURCE] => &[typed::<bmp::Report>("report")], limits(STRUCTURE, MIB, None, None, None), None,
        "forensics::formats::bmp::inspect", "inspect_bmp", "https://learn.microsoft.com/en-us/windows/win32/gdi/bitmap-storage",
        r#"{"inputs":{"data":{"base64":"Qk06AAAAAAAAADYAAAAoAAAAAQAAAP////8BABgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="}},"parameters":{}}"# =>
        r#"{"report":{"width":1,"height":1,"top_down":true,"bits_per_pixel":24,"compression":"bi_rgb","declared_size":58,"row_stride_bytes":4,"raster":{"offset":54,"length":4},"color_table":null,"channel_masks":null,"logical_end":58,"trailing_bytes":0,"unsupported_features":[],"payload_verified":false}}"#),
    row!("file.webp.inspect", "webp-inspect", "file", "Inspect WebP", "file framing",
        "Inspect checked RIFF/WebP chunks, canvas dimensions and padding.", "bytes" => "json", NoOptions, &[],
        &[SOURCE] => &[typed::<webp::Report>("report")], limits(STRUCTURE, MIB, Some(10_000), None, None), None,
        "forensics::formats::webp::inspect", "inspect_webp", "https://www.rfc-editor.org/rfc/rfc9649.html",
        r#"{"inputs":{"data":{"base64":"UklGRhIAAABXRUJQVlA4TAUAAAAvAAAAAAA="}},"parameters":{}}"# =>
        r#"{"report":{"width":1,"height":1,"declared_size":18,"logical_end":26,"trailing_bytes":0,"chunks":[{"fourcc":"VP8L","payload":{"offset":20,"length":5},"padded_length":6}],"unsupported_features":[],"payload_verified":false}}"#),
    row!("file.tiff.inspect", "tiff-inspect", "file", "Inspect classic TIFF", "file framing",
        "Inspect a bounded classic TIFF directory graph and exact field ranges.", "bytes" => "json", NoOptions, &[],
        &[SOURCE] => &[typed::<tiff::Report>("report")], limits(STRUCTURE, 4 * MIB, Some(10_000), None, None), None,
        "forensics::formats::tiff::inspect", "inspect_tiff", "https://pdfa.org/wp-content/uploads/2020/07/TIFF6.pdf",
        r#"{"inputs":{"data":{"base64":"SUkqAAgAAAABAAABAwABAAAAEAAAAAAAAAA="}},"parameters":{}}"# =>
        r#"{"report":{"byte_order":"little","first":8,"directories":[{"range":{"offset":8,"length":18},"entries":[{"tag":256,"field_type":3,"value_count":1,"entry_range":{"offset":10,"length":12},"value_range":{"offset":18,"length":2},"inline_hex":"1000","offset_value_hex":null,"unsupported_type":false}],"next":null,"sub_ifds":[]}],"payload_verified":false}}"#),
    row!("file.executable.inspect", "executable-inspect", "file", "Inspect ELF or PE", "file framing",
        "Inspect bounded ELF32/64 or PE32/32+ headers and declared extents.", "bytes" => "json", NoOptions, &[],
        &[SOURCE] => &[typed::<executable::Report>("report")], limits(STRUCTURE, 4 * MIB, Some(4096), None, None), None,
        "forensics::formats::executable::inspect", "inspect_executable", "https://refspecs.linuxfoundation.org/elf/gabi4+/contents.html",
        r#"{"inputs":{"data":{"base64":"f0VMRgIBAQAAAAAAAAAAAAIAPgABAAAAiHdmVUQzIhEAAAAAAAAAAAAAAAAAAAAAAAAAAEAAOAAAAEAAAAAAAA=="}},"parameters":{}}"# =>
        r#"{"report":{"format":"elf64","byte_order":"little","architecture":"x86_64","machine":62,"entry_point":"0x1122334455667788","image_base":null,"main_header":{"offset":0,"length":64},"stub":null,"program_table":null,"section_table":null,"program_headers":[],"sections":[],"data_directories":[],"unsupported_features":["interpreter_and_dynamic_metadata","section_payloads"]}}"#),
    row!("file.mpeg_audio.inspect", "mpeg-audio-inspect", "file", "Inspect ID3 and MPEG audio", "file framing",
        "Inspect bounded ID3v2.3/v2.4 tags and MPEG Layer III frame ranges.", "bytes" => "json", NoOptions, &[],
        &[SOURCE] => &[typed::<mpeg::Report>("report")], limits(STRUCTURE, 4 * MIB, Some(10_000), None, None), None,
        "forensics::formats::mpeg::inspect", "inspect_mpeg_audio", "https://id3.org/id3v2.4.0-structure",
        r#"{"inputs":{"data":{"base64":"SUQzAwAAAAAAEFRJVDIAAAAGAAAAVGl0bGU="}},"parameters":{}}"# =>
        r#"{"report":{"tag":{"version":"2.3","range":{"offset":0,"length":26},"extended_header":null,"footer":null,"unsynchronized":false,"frames":[{"id":"TIT2","range":{"offset":10,"length":16},"payload":{"offset":20,"length":6},"flags":0,"text":"Title","unsupported_payload":false}]},"frames":[],"trailing_id3v1":null,"payload_verified":false}}"#),
    row!("file.ogg.inspect", "ogg-inspect", "file", "Inspect Ogg pages", "file framing",
        "Verify RFC 3533 page CRCs and report Ogg streams and packet-fragment ranges.", "bytes" => "json", NoOptions, &[],
        &[SOURCE] => &[typed::<ogg::Report>("report")], limits(STRUCTURE, 4 * MIB, Some(10_000), WORK, None), None,
        "forensics::formats::ogg::inspect", "inspect_ogg", "https://www.rfc-editor.org/rfc/rfc3533.html",
        r#"{"inputs":{"data":{"base64":"T2dnUwAGAAAAAAAAAAAHAAAAAAAAAJaPlD8BAWE="}},"parameters":{}}"# =>
        r#"{"report":{"pages":[{"range":{"offset":0,"length":29},"serial":7,"sequence":0,"flags":6,"granule_position":"0","checksum":1066700694,"lacing":[1],"fragments":[{"packet":0,"range":{"offset":28,"length":1},"complete":true}]}],"streams":[{"serial":7,"pages":1,"packets":1,"ended":true}],"checksum_verified":true}}"#),
];

const fn contracts() -> [c::Entry; ROWS.len()] {
    let mut entries = [ROWS[0].contract; ROWS.len()];
    let mut i = 1;
    while i < ROWS.len() {
        entries[i] = ROWS[i].contract;
        i += 1;
    }
    entries
}
const fn metadata() -> [m::Entry; ROWS.len()] {
    let mut entries = [m::Entry {
        descriptor: ROWS[0].contract.descriptor,
        metadata: None,
    }; ROWS.len()];
    let mut i = 0;
    while i < ROWS.len() {
        entries[i] = m::Entry {
            descriptor: ROWS[i].contract.descriptor,
            metadata: Some(ROWS[i].metadata),
        };
        i += 1;
    }
    entries
}

pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&metadata());
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &metadata()).with_contracts(&contracts());

/// Resolve an explicit command name through the owning declarations.
pub fn command(slug: &str) -> Option<&'static str> {
    OPERATIONS
        .iter()
        .find(|row| row.command.strip_prefix("binary format ") == Some(slug))
        .map(|row| row.id)
}
