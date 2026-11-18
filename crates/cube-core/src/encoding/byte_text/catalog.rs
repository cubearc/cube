//! Stable byte-text profiles and typed invocation contracts.
use super::Profile;
use crate::{
    errors::OperationError,
    operations::{self as o, contracts as c, metadata as m},
};
use serde_json::json;

struct Row {
    encoding: m::Entry,
    decoding: m::Entry,
}

#[allow(clippy::too_many_arguments)]
const fn row(
    ids: [&'static str; 2],
    name: &'static str,
    description: &'static str,
    encode_command: &'static str,
    decode_command: &'static str,
    examples: [&'static str; 2],
    api: [&'static str; 2],
    python: [&'static str; 2],
    references: &'static [&'static str],
    reversible_decode: bool,
) -> Row {
    let descriptor = o::Descriptor {
        id: ids[0],
        category: "encoding",
        description,
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(true),
        command: encode_command,
        example: examples[0],
    };
    let metadata = m::Metadata {
        schema_version: 1,
        canonical_name: name,
        family: "byte_encodings",
        aliases: &[],
        security: m::Security {
            classification: m::Classification::Encoding,
            limitations: "Reversible encoding without confidentiality or authentication.",
            private_input: false,
            private_output: false,
            sensitive_parameters: &[],
        },
        profile: m::Profile {
            variant: description,
            alphabet: "Exact bytes and the named ASCII representation.",
            key: "No key.",
            key_advancement: "Not applicable.",
            block_bits: None,
            output_bits: None,
        },
        resources: &[],
        resource_notes: "See the operation contract for directional byte ceilings. Text is interpreted exactly.",
        rust_api: api[0],
        python_api: python[0],
        references,
    };
    Row {
        encoding: m::Entry {
            descriptor,
            metadata: Some(metadata),
        },
        decoding: m::Entry {
            descriptor: o::Descriptor {
                id: ids[1],
                input_type: "text",
                output_type: "bytes",
                command: decode_command,
                example: examples[1],
                reversible: Some(reversible_decode),
                ..descriptor
            },
            metadata: Some(m::Metadata {
                rust_api: api[1],
                python_api: python[1],
                ..metadata
            }),
        },
    }
}

const fn pair(encode: &'static str, decode: &'static str) -> [&'static str; 2] {
    [encode, decode]
}
const fn direction(id: &'static str) -> &'static str {
    id
}
const ROWS: &[Row] = &[
    row(
        pair(
            "encoding.binary.encode",
            direction("encoding.binary.decode"),
        ),
        "Binary byte tokens",
        "Eight binary digits per byte with one ASCII space between tokens.",
        "encode binary",
        "decode binary",
        [
            "cube encode binary --text 'foo'",
            "cube decode binary --text '01100110 01101111 01101111'",
        ],
        [
            "cube_core::encoding::byte_text::binary_encode",
            "cube_core::encoding::byte_text::binary_decode",
        ],
        ["binary_encode", "binary_decode"],
        &["https://doc.rust-lang.org/std/fmt/index.html"],
        true,
    ),
    row(
        pair("encoding.octal.encode", direction("encoding.octal.decode")),
        "Octal byte tokens",
        "Three octal digits per byte with one ASCII space between tokens. Values above 377 fail.",
        "encode octal",
        "decode octal",
        [
            "cube encode octal --text 'foo'",
            "cube decode octal --text '146 157 157'",
        ],
        [
            "cube_core::encoding::byte_text::octal_encode",
            "cube_core::encoding::byte_text::octal_decode",
        ],
        ["octal_encode", "octal_decode"],
        &["https://doc.rust-lang.org/std/fmt/index.html"],
        true,
    ),
    row(
        pair(
            "encoding.base36.encode",
            direction("encoding.base36.decode"),
        ),
        "Base36 bytes",
        "Lowercase GMP magnitude alphabet with one leading zero digit per leading zero byte.",
        "encode base36",
        "decode base36",
        [
            "cube encode base36 --text 'foo'",
            "cube decode base36 --text '3zvxr'",
        ],
        [
            "cube_core::encoding::byte_text::base36_encode",
            "cube_core::encoding::byte_text::base36_decode",
        ],
        ["base36_encode", "base36_decode"],
        &["https://gmplib.org/manual/Converting-Integers"],
        true,
    ),
    row(
        pair(
            "encoding.base62.encode",
            direction("encoding.base62.decode"),
        ),
        "Base62 bytes",
        "GMP digits, uppercase letters, then lowercase letters, with one leading zero digit per leading zero byte.",
        "encode base62",
        "decode base62",
        [
            "cube encode base62 --text 'foo'",
            "cube decode base62 --text 'SAPP'",
        ],
        [
            "cube_core::encoding::byte_text::base62_encode",
            "cube_core::encoding::byte_text::base62_decode",
        ],
        ["base62_encode", "base62_decode"],
        &["https://gmplib.org/manual/Converting-Integers"],
        true,
    ),
    row(
        pair(
            "encoding.ascii85.encode",
            direction("encoding.ascii85.decode"),
        ),
        "Framed Adobe Ascii85",
        "Explicit Adobe framing with canonical zero shorthand and final group.",
        "encode ascii85",
        "decode ascii85",
        [
            "cube encode ascii85 --text 'foo'",
            "cube decode ascii85 --text '<~AoDS~>'",
        ],
        [
            "cube_core::encoding::byte_text::ascii85_encode",
            "cube_core::encoding::byte_text::ascii85_decode",
        ],
        ["ascii85_encode", "ascii85_decode"],
        &["https://www.adobe.com/content/dam/acom/en/devnet/pdf/pdfs/PDF32000_2008.pdf"],
        true,
    ),
    row(
        pair(
            "encoding.ascii85_bare.encode",
            direction("encoding.ascii85_bare.decode"),
        ),
        "Bare Adobe Ascii85",
        "Adobe packing with canonical zero shorthand and final group, without framing.",
        "encode ascii85-bare",
        "decode ascii85-bare",
        [
            "cube encode ascii85-bare --text 'foo'",
            "cube decode ascii85-bare --text 'AoDS'",
        ],
        [
            "cube_core::encoding::byte_text::ascii85_bare_encode",
            "cube_core::encoding::byte_text::ascii85_bare_decode",
        ],
        ["ascii85_bare_encode", "ascii85_bare_decode"],
        &["https://www.adobe.com/content/dam/acom/en/devnet/pdf/pdfs/PDF32000_2008.pdf"],
        true,
    ),
    row(
        pair(
            "encoding.base91.encode",
            direction("encoding.base91.decode"),
        ),
        "Original basE91",
        "Original basE91 alphabet and 13/14-bit packing with a strict canonical tail.",
        "encode base91",
        "decode base91",
        [
            "cube encode base91 --text 'This is a random binary string'",
            "cube decode base91 --text 'nX,<:WRT%yxtYQ:mbr4JB*H[LR/@Qj{o0aU=Z'",
        ],
        [
            "cube_core::encoding::byte_text::base91_encode",
            "cube_core::encoding::byte_text::base91_decode",
        ],
        ["base91_encode", "base91_decode"],
        &["https://docs.rs/base91/0.1.0/base91/"],
        true,
    ),
    row(
        pair(
            "encoding.base92.encode",
            direction("encoding.base92.decode"),
        ),
        "Original Base92",
        "Original Base92 alphabet and 13-bit packing, canonical tail and tilde empty marker.",
        "encode base92",
        "decode base92",
        [
            "cube encode base92 --text 'hello world'",
            "cube decode base92 --text 'Fc_$aOTdKnsM*k'",
        ],
        [
            "cube_core::encoding::byte_text::base92_encode",
            "cube_core::encoding::byte_text::base92_decode",
        ],
        ["base92_encode", "base92_decode"],
        &["https://github.com/thenoviceoof/base92"],
        true,
    ),
    row(
        pair(
            "encoding.url_component.encode",
            direction("encoding.url_component.decode"),
        ),
        "RFC 3986 component",
        "Unreserved ASCII or percent triplets. Encoding emits uppercase triplets. Decoding accepts either hex case.",
        "encode url-component",
        "decode url-component",
        [
            "cube encode url-component --text 'foo'",
            "cube decode url-component --text 'foo'",
        ],
        [
            "cube_core::encoding::byte_text::url_component_encode",
            "cube_core::encoding::byte_text::url_component_decode",
        ],
        ["url_component_encode", "url_component_decode"],
        &["https://www.rfc-editor.org/rfc/rfc3986"],
        false,
    ),
];
const COUNT: usize = ROWS.len() * 2;
const fn flatten() -> [m::Entry; COUNT] {
    let mut result = [ROWS[0].encoding; COUNT];
    let mut i = 0;
    while i < ROWS.len() {
        result[i * 2] = ROWS[i].encoding;
        result[i * 2 + 1] = ROWS[i].decoding;
        i += 1;
    }
    result
}
const PROFILED: [m::Entry; COUNT] = flatten();

pub(super) const fn id(profile: Profile, encode: bool) -> &'static str {
    PROFILED[2 * profile as usize + !encode as usize]
        .descriptor
        .id
}
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct EmptyParameters {}

const fn contract(
    profile: Profile,
    encode: bool,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    let bytes = profile.byte_limit() as u64;
    let text = profile.text_limit() as u64;
    c::Entry {
        operation_version: 1,
        descriptor: PROFILED[profile as usize * 2 + !encode as usize].descriptor,
        inputs,
        parameters: c::schema::<EmptyParameters>,
        outputs,
        limits: c::Limits {
            max_input_bytes: Some(if encode { bytes } else { text }),
            max_output_bytes: Some(if encode { text } else { bytes }),
            max_items: None,
            max_work: Some(match profile {
                Profile::Base36 | Profile::Base62 => text * text,
                _ => text * 2,
            }),
            max_memory_bytes: None,
        },
        fixture,
    }
}
macro_rules! byte_text_contracts {
    ($(($profile:ident, $plain:literal, $wire:literal)),+ $(,)?) => {
        const CONTRACTS: &[c::Entry] = &[$(
            contract(Profile::$profile, true,
                &[c::Port { name: "input", semantic_type: "bytes", required: true, schema: c::bytes_schema::<{ Profile::$profile.byte_limit() }> }],
                &[c::Port { name: "output", semantic_type: "text", required: true, schema: c::text_schema::<{ Profile::$profile.text_limit() }> }],
                || Ok(c::Fixture {
                    inputs: json!({"input": {"base64": crate::encoding::base64_encode($plain)}}),
                    parameters: json!({}),
                    outputs: json!({"output": Profile::$profile.encode($plain)?}),
                }),
            ),
            contract(Profile::$profile, false,
                &[c::Port { name: "input", semantic_type: "text", required: true, schema: c::text_schema::<{ Profile::$profile.text_limit() }> }],
                &[c::Port { name: "output", semantic_type: "bytes", required: true, schema: c::bytes_schema::<{ Profile::$profile.byte_limit() }> }],
                || Ok(c::Fixture {
                    inputs: json!({"input": $wire}),
                    parameters: json!({}),
                    outputs: json!({"output": {"base64": crate::encoding::base64_encode(&Profile::$profile.decode($wire)?)}}),
                }),
            ),
        )+];
    };
}

byte_text_contracts![
    (Binary, b"foo", "01100110 01101111 01101111"),
    (Octal, b"foo", "146 157 157"),
    (Base36, b"foo", "3zvxr"),
    (Base62, b"foo", "SAPP"),
    (Ascii85, b"foo", "<~AoDS~>"),
    (Ascii85Bare, b"foo", "AoDS"),
    (
        Base91,
        b"This is a random binary string",
        "nX,<:WRT%yxtYQ:mbr4JB*H[LR/@Qj{o0aU=Z"
    ),
    (Base92, b"hello world", "Fc_$aOTdKnsM*k"),
    (UrlComponent, b"foo", "foo"),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
