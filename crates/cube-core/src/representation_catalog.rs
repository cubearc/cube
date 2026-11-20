//! Versioned named-port contracts for binary representations and fingerprints.
use crate::{
    binary::{bcd, bit_units, reed_solomon, representation, varint},
    encoding::rfc4648,
    hashing::fingerprints,
    operations::{self as o, contracts as c, metadata as m},
};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const MIB: u64 = 1 << 20;
pub(crate) const BLOCK_LIMIT: usize = 4096;
pub const MAX_DOCUMENT: usize = 24 << 20;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NoParameters {}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BlockBytes {
    #[schemars(length(max = 340), extend("contentEncoding" = "base64"))]
    pub base64: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct Erased {
    pub codeword: BlockBytes,
    #[schemars(length(max = 64))]
    pub erasures: Vec<u8>,
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct Encoded {
    pub parity: c::values::Bytes,
    pub codeword: c::values::Bytes,
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct Corrected {
    pub message: c::values::Bytes,
    pub codeword: c::values::Bytes,
    pub positions: Vec<usize>,
    pub initial_syndromes: Vec<u8>,
    pub verified_syndromes: Vec<u8>,
}

fn array<T: JsonSchema, const MAX: usize>() -> Schema {
    let mut schema = c::schema::<Vec<T>>().to_value();
    schema["maxItems"] = json!(MAX);
    Schema::try_from(schema).expect("bounded representation array")
}
fn sip_key() -> Schema {
    let mut schema = c::bytes_schema::<16>().to_value();
    schema["properties"]["base64"]["minLength"] = json!(24);
    schema["properties"]["base64"]["pattern"] = json!("^[A-Za-z0-9+/]{21}[AQgw]==$");
    schema["x-cube-secret"] = json!(true);
    Schema::try_from(schema).expect("exact SipHash key transport")
}

fn sip_digest() -> Schema {
    let mut schema = c::schema::<fingerprints::Digest>().to_value();
    schema["properties"]["hex"]["pattern"] = json!("^[0-9a-f]{16}$");
    Schema::try_from(schema).expect("SipHash digest transport")
}

fn fingerprint_hex() -> Schema {
    schemars::json_schema!({"type":"string","pattern":"^(?:[0-9a-f]{8}|[0-9a-f]{16}|[0-9a-f]{32})$"})
}

const fn port(name: &'static str, kind: &'static str, schema: fn() -> Schema) -> c::Port {
    c::Port {
        name,
        semantic_type: kind,
        schema,
        required: true,
    }
}
const fn bytes<const MAX: usize>(name: &'static str) -> c::Port {
    port(name, "bytes", c::bytes_schema::<MAX>)
}
const fn text<const MAX: usize>(name: &'static str) -> c::Port {
    port(name, "text", c::text_schema::<MAX>)
}
const fn typed<T: JsonSchema>(name: &'static str) -> c::Port {
    port(name, "json", c::schema::<T>)
}
const fn items<T: JsonSchema, const MAX: usize>(name: &'static str) -> c::Port {
    port(name, "json", array::<T, MAX>)
}
const fn limits(input: u64, output: u64, count: Option<u64>) -> c::Limits {
    c::Limits {
        max_input_bytes: Some(input),
        max_output_bytes: Some(output),
        max_items: count,
        max_work: None,
        max_memory_bytes: None,
    }
}
const RS_LIMITS: c::Limits = c::Limits {
    max_input_bytes: Some(MIB),
    max_output_bytes: Some(4 * MIB),
    max_items: Some(4096),
    max_work: Some(16 * MIB),
    max_memory_bytes: Some(16 * MIB),
};
const BIT_LIMITS: c::Limits = c::Limits {
    max_work: Some(8 * MIB),
    ..limits(MIB, 8 * MIB, None)
};
const INTEGERS: c::Limits = limits(MIB, MIB, Some(65_536));

const SIP_DESCRIPTION: &str =
    "Compute an exact little-endian SipHash-2-4 digest with a caller-provided private key.";
const SIPHASH: m::Metadata = m::Metadata {
    schema_version: 1,
    canonical_name: "SipHash-2-4",
    family: "keyed digest",
    aliases: &[],
    security: m::Security {
        classification: m::Classification::PrivateMaterial,
        private_input: true,
        private_output: true,
        sensitive_parameters: &[],
        limitations: "64-bit keyed digest for the documented SipHash profile. Treat the caller's key and result as private.",
    },
    profile: m::Profile {
        variant: "",
        alphabet: "raw bytes",
        key: "required private 16-byte raw input",
        key_advancement: "two compression rounds and four finalization rounds",
        block_bits: Some(64),
        output_bits: Some(64),
    },
    resources: &[m::ResourceBound {
        name: "message",
        maximum: 16 * MIB,
        unit: "bytes",
        scope: m::Scope::Core,
    }],
    resource_notes: "Direct invocation with private input and output; no automatic capture.",
    rust_api: "cube_core::hashing::fingerprints::siphash24",
    python_api: "siphash24",
    references: &["https://github.com/veorq/SipHash"],
};

const HASH_PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Explicit fingerprint profile",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["xxh32", "xxh64", "murmur3_x86_32", "murmur3_x64_128"],
        },
    },
    o::Parameter {
        name: "seed",
        description: "Pipeline seed as canonical unsigned decimal text, 32 bits or 64 bits for XXH64",
        required: false,
        default: Some(o::ParameterValue::Text("0")),
        constraints: o::ParameterType::Text { choices: &[] },
    },
];
const URL_PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "padding",
    description: "Require canonical padding or forbid padding",
    required: false,
    default: Some(o::ParameterValue::Text("required")),
    constraints: o::ParameterType::Text {
        choices: &["required", "forbidden"],
    },
}];
const fn parameterized(mut row: Row, parameters: &'static [o::Parameter]) -> Row {
    row.contract.descriptor.parameters = parameters;
    row
}

struct Row {
    contract: c::Entry,
    metadata: m::Metadata,
}
const fn declare(id: &'static str, mut row: Row) -> Row {
    row.contract.descriptor.id = id;
    row.metadata.profile.variant = row.contract.descriptor.description;
    row
}

macro_rules! public_profile {
    ($name:literal, $family:literal, $class:ident, $alphabet:literal, $limitations:literal,
     $notes:literal, $rust:literal, $py:literal, [$($reference:literal),+]) => {
        m::Metadata {
            schema_version: 1,
            canonical_name: $name,
            family: $family,
            aliases: &[],
            security: m::Security {
                classification: m::Classification::$class,
                limitations: $limitations,
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: "",
                alphabet: $alphabet,
                key: "none",
                key_advancement: "none",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: $notes,
            rust_api: concat!("cube_core::", $rust),
            python_api: $py,
            references: &[$($reference),+],
        }
    };
}

macro_rules! row {
    ($cat:literal, $slug:literal, $kind:literal => $output:literal, $desc:expr,
     $params:ty, $inputs:expr => $outputs:expr, $limits:expr, $reversible:expr,
     $sample:literal => $answer:literal, $metadata:expr) => {
        Row {
            metadata: $metadata,
            contract: c::Entry {
                operation_version: 1,
                descriptor: o::Descriptor {
                    id: "",
                    category: $cat,
                    description: $desc,
                    input_type: $kind,
                    output_type: $output,
                    parameters: &[],
                    deterministic: true,
                    reversible: $reversible,
                    command: concat!("binary format ", $slug),
                    example: concat!("cube binary format ", $slug, " --text '", $sample, "'"),
                },
                inputs: $inputs,
                parameters: c::schema::<$params>,
                outputs: $outputs,
                limits: $limits,
                fixture: || {
                    let mut sample: Value =
                        serde_json::from_str($sample).expect("representation fixture");
                    Ok(c::Fixture {
                        inputs: sample["inputs"].take(),
                        parameters: sample["parameters"].take(),
                        outputs: serde_json::from_str($answer)
                            .expect("representation fixture answer"),
                    })
                },
            },
        }
    };
}

const ROWS: [Row; 19] = [
    declare(
        "binary.pack_bits",
        row!("binary", "pack-bits", "bytes" => "json",
        "Pack exact byte-valued bits in the specified order and report their count.",
        bit_units::Pack, &[bytes::<1048576>("data")] => &[bytes::<131072>("output"), typed::<usize>("valid_bits")],
        BIT_LIMITS, Some(true),
        r#"{"inputs":{"data":{"base64":"AQAAAAAAAAEB"}},"parameters":{"bit_order":"msb","tail":"pad_zero"}}"#
        => r#"{"output":{"base64":"gYA="},"valid_bits":9}"#,
        public_profile!("Pack bits", "bit representations", BinaryTransform, "byte-valued bits 0 and 1", "Uses the explicit bit order and can represent a partial final byte.", "Up to 1 MiB input and 128 KiB packed data plus the valid-bit count.", "binary::bit_units::pack", "pack_bits", ["https://github.com/cubearc/cube/issues/385"])),
    ),
    declare(
        "binary.unpack_bits",
        row!("binary", "unpack-bits", "bytes" => "json",
        "Expand a declared count of packed bits to exact bytes valued zero or one.",
        bit_units::Unpack, &[bytes::<1048576>("data")] => &[bytes::<8388608>("output"), typed::<usize>("valid_bits")],
        BIT_LIMITS, Some(true),
        r#"{"inputs":{"data":{"base64":"gYA="}},"parameters":{"bit_order":"msb","valid_bits":9}}"#
        => r#"{"output":{"base64":"AQAAAAAAAAEB"},"valid_bits":9}"#,
        public_profile!("Unpack bits", "bit representations", BinaryTransform, "exact bytes", "Uses the explicit bit order and can represent a partial final byte.", "Up to 1 MiB packed input and 8 MiB expanded bytes plus the valid-bit count.", "binary::bit_units::unpack", "unpack_bits", ["https://github.com/cubearc/cube/issues/385"])),
    ),
    declare(
        "binary.mask",
        row!("binary", "mask", "bytes" => "bytes",
        "Repeat one unsigned AND mask across complete big-endian units.",
        bit_units::Mask, &[bytes::<16777216>("data")] => &[bytes::<16777216>("output")],
        limits(16 * MIB, 16 * MIB, None), Some(false),
        r#"{"inputs":{"data":{"base64":"//8SNA=="}},"parameters":{"unit_bytes":2,"mask":255}}"#
        => r#"{"output":{"base64":"AP8ANA=="}}"#,
        public_profile!("Fixed-unit bit mask", "bit operations", BinaryTransform, "exact bytes", "Uses independent unsigned big-endian units of 1, 2, 4 or 8 bytes.", "Up to 16 MiB of input and output bytes.", "binary::bit_units::mask", "mask_bytes", ["https://github.com/cubearc/cube/issues/386"])),
    ),
    declare(
        "binary.count_bits",
        row!("binary", "count-bits", "bytes" => "json",
        "Report population, leading-zero or trailing-zero counts per complete big-endian unit.",
        bit_units::Count, &[bytes::<16777216>("data")] => &[items::<u32, 65536>("counts")],
        limits(16 * MIB, MIB, Some(65_536)), Some(false),
        r#"{"inputs":{"data":{"base64":"//8SNA=="}},"parameters":{"unit_bytes":2,"kind":"population"}}"#
        => r#"{"counts":[16,5]}"#,
        public_profile!("Fixed-unit bit counts", "bit operations", BinaryTransform, "exact bytes", "Uses independent unsigned big-endian units of 1, 2, 4 or 8 bytes.", "Up to 16 MiB of input and 65,536 ordered counts.", "binary::bit_units::count", "count_bits", ["https://github.com/cubearc/cube/issues/386"])),
    ),
    declare(
        "binary.fixed_integer.encode",
        row!("binary", "fixed-integer-encode", "json" => "bytes",
        "Serialize exact signed or unsigned integers with checked narrowing and explicit byte order.",
        representation::IntegerFormat, &[items::<representation::Integer, 65536>("values")] => &[bytes::<1048576>("output")],
        INTEGERS, Some(true),
        r#"{"inputs":{"values":[1,"18446744073709551615"]},"parameters":{"bits":64,"byte_order":"little","signedness":"unsigned"}}"#
        => r#"{"output":{"base64":"AQAAAAAAAAD//////////w=="}}"#,
        public_profile!("Encode fixed-width integers", "integer representations", BinaryTransform, "exact 64-bit integer values", "Checked narrowing and explicit byte order at all four widths.", "Up to 65,536 values and 1 MiB of transport.", "binary::representation::encode", "fixed_integer_encode", ["https://github.com/cubearc/cube/issues/387"])),
    ),
    declare(
        "binary.fixed_integer.decode",
        row!("binary", "fixed-integer-decode", "bytes" => "json",
        "Decode complete fixed-width integer units with exact width, signedness and byte order.",
        representation::IntegerFormat, &[bytes::<1048576>("data")] => &[typed::<representation::Integers>("result")],
        INTEGERS, Some(true),
        r#"{"inputs":{"data":{"base64":"AQAAAAAAAAD//////////w=="}},"parameters":{"bits":64,"byte_order":"little","signedness":"unsigned"}}"#
        => r#"{"result":{"signedness":"unsigned","values":[1,"18446744073709551615"]}}"#,
        public_profile!("Decode fixed-width integers", "integer representations", BinaryTransform, "exact bytes", "Checked narrowing and explicit byte order at all four widths.", "Up to 65,536 integers and 1 MiB of report bytes.", "binary::representation::decode", "fixed_integer_decode", ["https://github.com/cubearc/cube/issues/387"])),
    ),
    declare(
        "binary.bcd.encode",
        row!("binary", "bcd-encode", "json" => "bytes",
        "Encode exact width-preserving decimal digits using an explicit BCD profile.",
        bcd::Layout, &[text::<4096>("digits"), typed::<bool>("negative")] => &[bytes::<16384>("output")],
        limits(4096, 16384, Some(4096)), None,
        r#"{"inputs":{"digits":"123","negative":true},"parameters":{"profile":"signed_packed","digits":3}}"#
        => r#"{"output":{"base64":"Ej0="}}"#,
        public_profile!("Encode binary-coded decimal", "binary-coded decimal", BinaryTransform, "ASCII digits and declared decimal sign", "Uses exact digit widths and the selected sign and padding profile.", "Up to 4,096 digits and 16 KiB output.", "binary::bcd::encode", "bcd_encode", ["https://www.ibm.com/docs/en/cobol-zos/6.4.0?topic=arithmetic-formats-numeric-data"])),
    ),
    declare(
        "binary.bcd.decode",
        row!("binary", "bcd-decode", "bytes" => "json",
        "Read and validate packed or unpacked BCD while preserving all declared digits.",
        bcd::Decode, &[bytes::<16384>("data")] => &[typed::<bcd::Decimal>("result")],
        limits(16384, 16384, Some(4096)), None,
        r#"{"inputs":{"data":{"base64":"Ej0="}},"parameters":{"profile":"signed_packed","digits":3}}"#
        => r#"{"result":{"digits":"123","negative":true}}"#,
        public_profile!("Decode binary-coded decimal", "binary-coded decimal", BinaryTransform, "exact bytes", "Uses exact digit widths and the selected sign and padding profile.", "Up to 16 KiB input and 4,096 output digits.", "binary::bcd::decode", "bcd_decode", ["https://www.ibm.com/docs/en/cobol-zos/6.4.0?topic=arithmetic-formats-numeric-data"])),
    ),
    declare(
        "binary.varint.encode",
        row!("binary", "varint-encode", "json" => "bytes",
        "Encode canonical ULEB128, SLEB128, Protobuf uint64 or ZigZag64 varints.",
        varint::Encode, &[items::<representation::Integer, 65536>("values")] => &[bytes::<1048576>("output")],
        INTEGERS, Some(true),
        r#"{"inputs":{"values":[624485]},"parameters":{"profile":"uleb128"}}"#
        => r#"{"output":{"base64":"5Y4m"}}"#,
        public_profile!("Encode variable-length integers", "variable-length integers", BinaryTransform, "exact 64-bit integer values", "Uses at most ten bytes per unsigned or signed 64-bit value.", "Up to 65,536 values and 1 MiB bytes or transport.", "binary::varint::encode", "varint_encode", ["https://dwarfstd.org/doc/DWARF5.pdf", "https://protobuf.dev/programming-guides/encoding/"])),
    ),
    declare(
        "binary.varint.decode",
        row!("binary", "varint-decode", "bytes" => "json",
        "Decode a varint prefix, one complete value or a batch and report consumed byte counts.",
        varint::Decode, &[bytes::<1048576>("data")] => &[items::<varint::Record, 65536>("records")],
        INTEGERS, None,
        r#"{"inputs":{"data":{"base64":"5Y4m"}},"parameters":{"profile":"uleb128","mode":"batch"}}"#
        => r#"{"records":[{"value":624485,"consumed":3}]}"#,
        public_profile!("Decode variable-length integers", "variable-length integers", BinaryTransform, "exact bytes", "Uses at most ten bytes per unsigned or signed 64-bit value.", "Up to 65,536 records and 1 MiB of input bytes or reports.", "binary::varint::decode", "varint_decode", ["https://dwarfstd.org/doc/DWARF5.pdf", "https://protobuf.dev/programming-guides/encoding/"])),
    ),
    declare(
        "binary.ieee754.inspect",
        row!("binary", "ieee754-inspect", "bytes" => "json",
        "Inspect exact binary32 or binary64 fields, including signed zero and NaN payloads.",
        representation::FloatLayout, &[bytes::<524288>("data")] => &[items::<representation::FloatRecord, 65536>("records")],
        limits(524288, 4 * MIB, Some(65_536)), Some(false),
        r#"{"inputs":{"data":{"base64":"gAAAAH+AAAE="}},"parameters":{"format":"binary32","byte_order":"big"}}"#
        => r#"{"records":[{"bits":2147483648,"sign":true,"biased_exponent":0,"fraction":0,"classification":"zero"},{"bits":2139095041,"sign":false,"biased_exponent":255,"fraction":1,"classification":"signaling_nan"}]}"#,
        public_profile!("Inspect IEEE 754 fields", "floating-point representations", BinaryTransform, "exact bytes", "Interprets raw binary32 and binary64 fields without floating-point arithmetic.", "Up to 65,536 complete records and 4 MiB of reports.", "binary::representation::inspect_floats", "ieee754_inspect", ["https://doc.rust-lang.org/std/primitive.f64.html#method.from_bits"])),
    ),
    declare(
        "binary.reed_solomon.encode",
        row!("binary", "reed-solomon-encode", "json" => "json",
        "Encode shortened systematic Reed–Solomon blocks over a qualified explicit GF(256) field.",
        reed_solomon::Options, &[items::<BlockBytes, BLOCK_LIMIT>("blocks")] => &[items::<Encoded, BLOCK_LIMIT>("blocks")],
        RS_LIMITS, Some(true),
        r#"{"inputs":{"blocks":[{"base64":"QFSEVMTE8OwR7BHsEewR7A=="}]},"parameters":{"field":{"profile":"qr-rs-gf256"},"parity_bytes":10}}"#
        => r#"{"blocks":[{"parity":{"base64":"I3MjmewIyfc33w=="},"codeword":{"base64":"QFSEVMTE8OwR7BHsEewR7CNzI5nsCMn3N98="}}]}"#,
        public_profile!("Encode Reed–Solomon blocks", "error-correcting codes", BinaryTransform, "exact bytes", "Error correction establishes codeword consistency; alternate valid codewords can exist.", "Up to 4,096 shortened blocks, 16 Mi field operations and 4 MiB of reports.", "binary::reed_solomon::encode", "reed_solomon_encode", ["https://www.nayuki.io/page/qr-code-generator-library"])),
    ),
    declare(
        "binary.reed_solomon.decode",
        row!("binary", "reed-solomon-decode", "json" => "json",
        "Correct shortened Reed–Solomon errors and erasures, then verify every output syndrome.",
        reed_solomon::Options, &[items::<Erased, BLOCK_LIMIT>("blocks")] => &[items::<Corrected, BLOCK_LIMIT>("blocks")],
        RS_LIMITS, None,
        r#"{"inputs":{"blocks":[{"codeword":{"base64":"QFSEf8TE8OwR7BHsEewR7CNzI5nsCMn3N/U="},"erasures":[25]}]},"parameters":{"field":{"profile":"qr-rs-gf256"},"parity_bytes":10}}"#
        => r#"{"blocks":[{"message":{"base64":"QFSEVMTE8OwR7BHsEewR7A=="},"codeword":{"base64":"QFSEVMTE8OwR7BHsEewR7CNzI5nsCMn3N98="},"positions":[3,25],"initial_syndromes":[1,6,170,26,32,224,200,199,104,251],"verified_syndromes":[0,0,0,0,0,0,0,0,0,0]}]}"#,
        public_profile!("Correct Reed–Solomon blocks", "error-correcting codes", BinaryTransform, "exact bytes", "Error correction establishes codeword consistency; alternate valid codewords can exist.", "Up to 4,096 shortened blocks, 16 Mi field operations and 4 MiB of reports.", "binary::reed_solomon::decode", "reed_solomon_decode", ["https://www.nayuki.io/page/qr-code-generator-library"])),
    ),
    declare(
        "hash.noncrypto",
        parameterized(
            row!("hashing", "noncrypto-hash", "bytes" => "text",
        "Compute seeded XXH32, XXH64 or a named MurmurHash3 digest in reference byte order.",
        fingerprints::Options, &[bytes::<16777216>("input")] => &[port("output", "text", fingerprint_hex)],
        limits(16 * MIB, 32, None), None,
        r#"{"inputs":{"input":{"base64":""}},"parameters":{"profile":"xxh64"}}"#
        => r#"{"output":"ef46db3751d8e999"}"#,
        public_profile!("Noncryptographic hash", "byte fingerprints", UnkeyedDigest, "exact bytes", "Noncryptographic indexing digest with explicitly selected seed width and byte order.", "Up to 16 MiB of input and 32 lowercase hexadecimal characters.", "hashing::fingerprints::noncrypto", "noncrypto_hash", ["https://github.com/Cyan4973/xxHash", "https://github.com/aappleby/smhasher"])),
            HASH_PARAMETERS,
        ),
    ),
    declare(
        "hash.siphash24",
        row!("hashing", "siphash24", "bytes" => "json",
        SIP_DESCRIPTION,
        NoParameters, &[bytes::<16777216>("data"), port("key", "bytes", sip_key)] => &[port("digest", "json", sip_digest)],
        limits(16 * MIB + 16, 48, None), None,
        r#"{"inputs":{"data":{"base64":""},"key":{"base64":"AAECAwQFBgcICQoLDA0ODw=="}},"parameters":{}}"#
        => r#"{"digest":{"hex":"310e0edd47db6f72"}}"#, SIPHASH),
    ),
    declare(
        "encoding.base32hex.encode",
        row!("encoding", "base32hex-encode", "bytes" => "text",
        "Encode canonical uppercase RFC 4648 Base32hex with padding.",
        NoParameters, &[bytes::<1048576>("input")] => &[text::<2097152>("output")],
        limits(MIB, 2 * MIB, None), Some(true),
        r#"{"inputs":{"input":{"base64":"Zm9v"}},"parameters":{}}"# => r#"{"output":"CPNMU==="}"#,
        public_profile!("Encode Base32hex", "RFC 4648", Encoding, "uppercase 0–9 and A–V with canonical padding", "Reversible encoding without confidentiality.", "Up to 1 MiB of bytes and 2 MiB of text.", "encoding::rfc4648::encode_base32hex", "base32hex_encode", ["https://www.rfc-editor.org/rfc/rfc4648.html"])),
    ),
    declare(
        "encoding.base32hex.decode",
        row!("encoding", "base32hex-decode", "text" => "bytes",
        "Decode strict padded RFC 4648 Base32hex with zero unused bits.",
        NoParameters, &[text::<2097152>("input")] => &[bytes::<1048576>("output")],
        limits(2 * MIB, MIB, None), Some(true),
        r#"{"inputs":{"input":"CPNMU==="},"parameters":{}}"# => r#"{"output":{"base64":"Zm9v"}}"#,
        public_profile!("Decode Base32hex", "RFC 4648", Encoding, "uppercase 0–9 and A–V with canonical padding", "Reversible encoding without confidentiality.", "Up to 2 MiB of text and 1 MiB of bytes.", "encoding::rfc4648::decode_base32hex", "base32hex_decode", ["https://www.rfc-editor.org/rfc/rfc4648.html"])),
    ),
    declare(
        "encoding.base64url.encode",
        parameterized(
            row!("encoding", "base64url-encode", "bytes" => "text",
        "Encode URL-safe RFC 4648 Base64 using the selected padding policy.",
        rfc4648::Base64Url, &[bytes::<1048576>("input")] => &[text::<2097152>("output")],
        limits(MIB, 2 * MIB, None), Some(true),
        r#"{"inputs":{"input":{"base64":"+/8="}},"parameters":{"padding":"forbidden"}}"# => r#"{"output":"-_8"}"#,
        public_profile!("Encode Base64url", "RFC 4648", Encoding, "URL-safe Base64 alphabet with required or forbidden padding", "Reversible encoding without confidentiality.", "Up to 1 MiB of bytes and 2 MiB of text.", "encoding::rfc4648::encode_base64url", "base64url_encode", ["https://www.rfc-editor.org/rfc/rfc4648.html"])),
            URL_PARAMETERS,
        ),
    ),
    declare(
        "encoding.base64url.decode",
        parameterized(
            row!("encoding", "base64url-decode", "text" => "bytes",
        "Decode strict URL-safe RFC 4648 Base64 with the exact selected padding policy.",
        rfc4648::Base64Url, &[text::<2097152>("input")] => &[bytes::<1048576>("output")],
        limits(2 * MIB, MIB, None), Some(true),
        r#"{"inputs":{"input":"-_8="},"parameters":{}}"# => r#"{"output":{"base64":"+/8="}}"#,
        public_profile!("Decode Base64url", "RFC 4648", Encoding, "URL-safe Base64 alphabet with required or forbidden padding", "Reversible encoding without confidentiality.", "Up to 2 MiB of text and 1 MiB of bytes.", "encoding::rfc4648::decode_base64url", "base64url_decode", ["https://www.rfc-editor.org/rfc/rfc4648.html"])),
            URL_PARAMETERS,
        ),
    ),
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
const fn profiles() -> [m::Entry; ROWS.len()] {
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
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&profiles());
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &profiles()).with_contracts(&contracts());

/// Resolve one CLI leaf using the owning declarations.
pub fn command(slug: &str) -> Option<&'static str> {
    OPERATIONS
        .iter()
        .find(|row| row.command.strip_prefix("binary format ") == Some(slug))
        .map(|row| row.id)
}
