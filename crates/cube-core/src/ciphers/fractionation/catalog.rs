//! One module-owned row projects descriptors, metadata and typed contracts.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use schemars::JsonSchema;
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[schemars(transparent)]
struct Text(#[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))] String);

#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
}
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "CLI-only bounded local JSON parameter file; canonical parameter fields are in the typed schema.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text { choices: &[] },
}];
struct Spec {
    name: &'static str,
    description: &'static str,
    reversible: bool,
    output_type: &'static str,
    id: &'static str,
    python: &'static str,
    rust: &'static str,
    command: &'static str,
    parameters: fn() -> schemars::Schema,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    fixture: fn() -> Result<c::Fixture, OperationError>,
    input_max: u64,
    output_max: u64,
    sensitive: &'static [&'static str],
    references: &'static [&'static str],
}
const fn row(s: Spec) -> Row {
    let Spec {
        name,
        description,
        reversible,
        output_type,
        id,
        python,
        rust,
        command,
        parameters,
        inputs,
        outputs,
        fixture,
        input_max,
        output_max,
        sensitive,
        references,
    } = s;
    let descriptor = o::Descriptor {
        id,
        category: "ciphers",
        description,
        input_type: "text",
        output_type,
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(reversible),
        command,
        example: "cube cipher --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs,
            parameters,
            outputs,
            limits: c::Limits {
                max_input_bytes: Some(input_max),
                max_output_bytes: Some(output_max),
                max_items: None,
                max_work: None,
                max_memory_bytes: None,
            },
            fixture,
        },
        profile: m::Entry {
            descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: name,
                family: "fractionation",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::HistoricalCipher,
                    limitations: "Historical/challenge use only; no modern confidentiality or authentication. Explicit aliases, source selection and padding evidence define the inverse.",
                    private_input: !sensitive.is_empty(),
                    private_output: !sensitive.is_empty(),
                    sensitive_parameters: sensitive,
                },
                profile: m::Profile {
                    variant: description,
                    alphabet: "Exact Unicode scalar permutations or explicit canonical coordinate/Morse alphabets; see profile guide.",
                    key: "Caller-owned strict typed fields; bounded permutations, masks, alphabets and schedules. No inferred parameters.",
                    key_advancement: "Per transformed letter or digraph/group according to the selected named profile; reset at each call.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "At most 65536 source symbols, 1 MiB text, 4096 columns/rails/period; ADFGX coordinate stream at most 131072. Strict parameter transport <=2 MiB. No implicit pad removal or uploads. Historical use only.",
                rust_api: rust,
                python_api: python,
                references,
            }),
        },
    }
}

#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct MorseStream(#[schemars(length(max = 1048576))] String);

#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct PadCount(#[schemars(range(min = 0, max = 2))] usize);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct PairPadCount(#[schemars(range(min = 0, max = 1))] usize);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct Tokens(#[schemars(length(max = 65536))] Vec<DecimalSum>);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct DecimalSum(#[schemars(range(min = 22, max = 110))] u16);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct NoPadding(#[schemars(range(min = 0, max = 0))] usize);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct Letters(
    #[schemars(length(max=65536),regex(pattern=r"^[A-Z]*$"),extend("x-cube-max-utf8-bytes"=65536))]
    String,
);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct SquareText(
    #[schemars(length(max=65536),regex(pattern=r"^[A-IK-Z]*$"),extend("x-cube-max-utf8-bytes"=65536))]
     String,
);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct Alphanumeric(
    #[schemars(length(max=65536),regex(pattern=r"^[A-Z0-9]*$"),extend("x-cube-max-utf8-bytes"=65536))]
     String,
);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct MorsePlain(
    #[schemars(length(max=65536),regex(pattern=r"^(?:[A-Z0-9]+(?: [A-Z0-9]+)*)?$"),extend("x-cube-max-utf8-bytes"=65536))]
     String,
);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct MorbitText(
    #[schemars(length(max=65536),regex(pattern=r"^[1-9]*$"),extend("x-cube-max-utf8-bytes"=65536))]
    String,
);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct PolluxText(
    #[schemars(length(max=65536),regex(pattern=r"^[0-9]*$"),extend("x-cube-max-utf8-bytes"=65536))]
    String,
);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct AdfgxText(
    #[schemars(length(max=131072),regex(pattern=r"^[ADFGX]*$"),extend("x-cube-max-utf8-bytes"=131072))]
     String,
);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct AdfgvxText(
    #[schemars(length(max=131072),regex(pattern=r"^[ADFGVX]*$"),extend("x-cube-max-utf8-bytes"=131072))]
     String,
);
const COUNT: usize = 22;
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct VicPlaintext(#[schemars(length(max = 65534), regex(pattern = r"^[A-Z.]*$"))] String);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct VicCiphertext(
    #[schemars(length(min = 10, max = 131080), regex(pattern = r"^[0-9]*$"))] String,
);
static ROWS: [Row; COUNT] = [
    row(Spec {
        name: "VIC (English 2010)",
        description: "Fixed 2010 English VIC: canonical A..Z/period text, explicit phrase/mnemonic/date/personal number, fixed50-digit derivation, ordinary then disrupted transposition, exact indicator/split/pad evidence. Date digit zero denotes group ten. No numeral-mode, Cyrillic, random padding or implicit normalization.",
        reversible: true,
        output_type: "text",
        id: "cipher.vic.encode",
        python: "vic_encode",
        rust: "cube_core::ciphers::fractionation::vic::transform",
        command: "cipher vic",
        parameters: c::schema::<vic::VicEnglish2010>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<VicPlaintext>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<VicCiphertext>,
            required: true,
        }],
        fixture: vic::fixtures::fixture_encode,
        input_max: 1048576,
        output_max: 1048576,
        sensitive: &[
            "phrase",
            "mnemonic",
            "date_digits",
            "personal_number",
            "message_indicator",
        ],
        references: &["https://raincomplex.net/writing/title/vic-cipher/"],
    }),
    row(Spec {
        name: "VIC (English 2010)",
        description: "Fixed 2010 English VIC: canonical A..Z/period text, explicit phrase/mnemonic/date/personal number, fixed50-digit derivation, ordinary then disrupted transposition, exact indicator/split/pad evidence. Date digit zero denotes group ten. No numeral-mode, Cyrillic, random padding or implicit normalization.",
        reversible: true,
        output_type: "text",
        id: "cipher.vic.decode",
        python: "vic_decode",
        rust: "cube_core::ciphers::fractionation::vic::transform",
        command: "cipher vic",
        parameters: c::schema::<vic::VicEnglish2010>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<VicCiphertext>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<VicPlaintext>,
            required: true,
        }],
        fixture: vic::fixtures::fixture_decode,
        input_max: 1048576,
        output_max: 1048576,
        sensitive: &[
            "phrase",
            "mnemonic",
            "date_digits",
            "personal_number",
            "message_indicator",
        ],
        references: &["https://raincomplex.net/writing/title/vic-cipher/"],
    }),
    row(Spec {
        name: "CM Bifid",
        description: "ACA conjugated-matrix Bifid with two explicit independent I/J squares and bounded periods; decode inverts the coordinate stream, not encryption with swapped squares.",
        reversible: true,
        output_type: "text",
        id: "cipher.cm_bifid.encode",
        python: "cm_bifid_encode",
        rust: "cube_core::ciphers::fractionation::cm::cm_bifid",
        command: "cipher cm-bifid",
        parameters: c::schema::<cm::CmBifid>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        fixture: cm::fixture_encode,
        input_max: 1048576,
        output_max: 1048576,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/CMBifid.pdf"],
    }),
    row(Spec {
        name: "CM Bifid",
        description: "ACA conjugated-matrix Bifid with two explicit independent I/J squares and bounded periods; decode inverts the coordinate stream, not encryption with swapped squares.",
        reversible: true,
        output_type: "text",
        id: "cipher.cm_bifid.decode",
        python: "cm_bifid_decode",
        rust: "cube_core::ciphers::fractionation::cm::cm_bifid",
        command: "cipher cm-bifid",
        parameters: c::schema::<cm::CmBifid>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        fixture: cm::fixture_decode,
        input_max: 1048576,
        output_max: 1048576,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/CMBifid.pdf"],
    }),
    row(Spec {
        name: "Bifid",
        description: "Bifid: explicit bounded encode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.bifid.encode",
        python: "bifid_encode",
        rust: "cube_core::ciphers::fractionation::bifid",
        command: "cipher bifid",
        parameters: c::schema::<Bifid>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Letters>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<SquareText>,
            required: true,
        }],
        fixture: fixtures::bifid_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Bifid.pdf"],
    }),
    row(Spec {
        name: "Bifid",
        description: "Bifid: explicit bounded decode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.bifid.decode",
        python: "bifid_decode",
        rust: "cube_core::ciphers::fractionation::bifid",
        command: "cipher bifid",
        parameters: c::schema::<Bifid>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Letters>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<SquareText>,
            required: true,
        }],
        fixture: fixtures::bifid_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Bifid.pdf"],
    }),
    row(Spec {
        name: "Trifid",
        description: "Trifid: explicit bounded encode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.trifid.encode",
        python: "trifid_encode",
        rust: "cube_core::ciphers::fractionation::trifid",
        command: "cipher trifid",
        parameters: c::schema::<Trifid>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        fixture: fixtures::trifid_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Trifid.pdf"],
    }),
    row(Spec {
        name: "Trifid",
        description: "Trifid: explicit bounded decode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.trifid.decode",
        python: "trifid_decode",
        rust: "cube_core::ciphers::fractionation::trifid",
        command: "cipher trifid",
        parameters: c::schema::<Trifid>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Text>,
            required: true,
        }],
        fixture: fixtures::trifid_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Trifid.pdf"],
    }),
    row(Spec {
        name: "Adfgx",
        description: "Adfgx: explicit bounded encode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.adfgx.encode",
        python: "adfgx_encode",
        rust: "cube_core::ciphers::fractionation::adfgx",
        command: "cipher adfgx",
        parameters: c::schema::<Adfgx>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Letters>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<AdfgxText>,
            required: true,
        }],
        fixture: fixtures::adfgx_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://legacy.cryptool.org/en/cto/adfg-v-x"],
    }),
    row(Spec {
        name: "Adfgx",
        description: "Adfgx: explicit bounded decode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.adfgx.decode",
        python: "adfgx_decode",
        rust: "cube_core::ciphers::fractionation::adfgx",
        command: "cipher adfgx",
        parameters: c::schema::<Adfgx>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<AdfgxText>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<SquareText>,
            required: true,
        }],
        fixture: fixtures::adfgx_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://legacy.cryptool.org/en/cto/adfg-v-x"],
    }),
    row(Spec {
        name: "Adfgvx",
        description: "Adfgvx: explicit bounded encode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.adfgvx.encode",
        python: "adfgvx_encode",
        rust: "cube_core::ciphers::fractionation::adfgvx",
        command: "cipher adfgvx",
        parameters: c::schema::<Adfgvx>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Alphanumeric>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<AdfgvxText>,
            required: true,
        }],
        fixture: fixtures::adfgvx_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://legacy.cryptool.org/en/cto/adfg-v-x"],
    }),
    row(Spec {
        name: "Adfgvx",
        description: "Adfgvx: explicit bounded decode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.adfgvx.decode",
        python: "adfgvx_decode",
        rust: "cube_core::ciphers::fractionation::adfgvx",
        command: "cipher adfgvx",
        parameters: c::schema::<Adfgvx>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<AdfgvxText>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Alphanumeric>,
            required: true,
        }],
        fixture: fixtures::adfgvx_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://legacy.cryptool.org/en/cto/adfg-v-x"],
    }),
    row(Spec {
        name: "Nihilist Substitution",
        description: "Nihilist Substitution: explicit bounded encode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "json",
        id: "cipher.nihilist_substitution.encode",
        python: "nihilist_substitution_encode",
        rust: "cube_core::ciphers::fractionation::nihilist",
        command: "cipher nihilist-substitution",
        parameters: c::schema::<Nihilist>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Letters>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "tokens",
            semantic_type: "json",
            schema: c::schema::<Tokens>,
            required: true,
        }],
        fixture: fixtures::nihilist_substitution_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &[
            "https://www.cryptogram.org/downloads/aca.info/ciphers/NihilistSubstitution.pdf",
        ],
    }),
    row(Spec {
        name: "Nihilist Substitution",
        description: "Nihilist Substitution: explicit bounded decode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.nihilist_substitution.decode",
        python: "nihilist_substitution_decode",
        rust: "cube_core::ciphers::fractionation::nihilist",
        command: "cipher nihilist-substitution",
        parameters: c::schema::<Nihilist>,
        inputs: &[c::Port {
            name: "tokens",
            semantic_type: "json",
            schema: c::schema::<Tokens>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<SquareText>,
            required: true,
        }],
        fixture: fixtures::nihilist_substitution_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &[
            "https://www.cryptogram.org/downloads/aca.info/ciphers/NihilistSubstitution.pdf",
        ],
    }),
    row(Spec {
        name: "Bazeries",
        description: "Bazeries: explicit bounded encode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.bazeries.encode",
        python: "bazeries_encode",
        rust: "cube_core::ciphers::fractionation::bazeries",
        command: "cipher bazeries",
        parameters: c::schema::<Bazeries>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Letters>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<SquareText>,
            required: true,
        }],
        fixture: fixtures::bazeries_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Bazeries.pdf"],
    }),
    row(Spec {
        name: "Bazeries",
        description: "Bazeries: explicit bounded decode; see typed profile guide for conventions.",
        reversible: true,
        output_type: "text",
        id: "cipher.bazeries.decode",
        python: "bazeries_decode",
        rust: "cube_core::ciphers::fractionation::bazeries",
        command: "cipher bazeries",
        parameters: c::schema::<Bazeries>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Letters>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<SquareText>,
            required: true,
        }],
        fixture: fixtures::bazeries_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Bazeries.pdf"],
    }),
    row(Spec {
        name: "Fractionated Morse",
        description: "Fractionated Morse: explicit bounded encode; see typed profile guide for conventions.",
        reversible: false,
        output_type: "json",
        id: "cipher.fractionated_morse.encode",
        python: "fractionated_morse_encode",
        rust: "cube_core::ciphers::fractionation::morse::fractionated_morse",
        command: "cipher fractionated-morse",
        parameters: c::schema::<morse::FractionatedMorse>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<MorsePlain>,
            required: true,
        }],
        outputs: &[
            c::Port {
                name: "text",
                semantic_type: "text",
                schema: c::schema::<Option<MorsePlain>>,
                required: true,
            },
            c::Port {
                name: "ciphertext",
                semantic_type: "text",
                schema: c::schema::<Letters>,
                required: true,
            },
            c::Port {
                name: "morse",
                semantic_type: "text",
                schema: c::schema::<MorseStream>,
                required: true,
            },
            c::Port {
                name: "padding_count",
                semantic_type: "json",
                schema: c::schema::<PadCount>,
                required: true,
            },
            c::Port {
                name: "padding_ambiguous",
                semantic_type: "json",
                schema: c::schema::<bool>,
                required: true,
            },
        ],
        fixture: fixtures::fractionated_morse_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &[
            "https://www.cryptogram.org/downloads/aca.info/ciphers/FractionatedMorse.pdf",
        ],
    }),
    row(Spec {
        name: "Fractionated Morse",
        description: "Fractionated Morse: explicit bounded decode; see typed profile guide for conventions.",
        reversible: false,
        output_type: "json",
        id: "cipher.fractionated_morse.decode",
        python: "fractionated_morse_decode",
        rust: "cube_core::ciphers::fractionation::morse::fractionated_morse",
        command: "cipher fractionated-morse",
        parameters: c::schema::<morse::FractionatedMorse>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<Letters>,
            required: true,
        }],
        outputs: &[
            c::Port {
                name: "text",
                semantic_type: "text",
                schema: c::schema::<Option<MorsePlain>>,
                required: true,
            },
            c::Port {
                name: "ciphertext",
                semantic_type: "text",
                schema: c::schema::<Letters>,
                required: true,
            },
            c::Port {
                name: "morse",
                semantic_type: "text",
                schema: c::schema::<MorseStream>,
                required: true,
            },
            c::Port {
                name: "padding_count",
                semantic_type: "json",
                schema: c::schema::<PadCount>,
                required: true,
            },
            c::Port {
                name: "padding_ambiguous",
                semantic_type: "json",
                schema: c::schema::<bool>,
                required: true,
            },
        ],
        fixture: fixtures::fractionated_morse_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &[
            "https://www.cryptogram.org/downloads/aca.info/ciphers/FractionatedMorse.pdf",
        ],
    }),
    row(Spec {
        name: "Morbit",
        description: "Morbit: explicit bounded encode; see typed profile guide for conventions.",
        reversible: false,
        output_type: "json",
        id: "cipher.morbit.encode",
        python: "morbit_encode",
        rust: "cube_core::ciphers::fractionation::morse::morbit",
        command: "cipher morbit",
        parameters: c::schema::<morse::Morbit>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<MorsePlain>,
            required: true,
        }],
        outputs: &[
            c::Port {
                name: "text",
                semantic_type: "text",
                schema: c::schema::<Option<MorsePlain>>,
                required: true,
            },
            c::Port {
                name: "ciphertext",
                semantic_type: "text",
                schema: c::schema::<MorbitText>,
                required: true,
            },
            c::Port {
                name: "morse",
                semantic_type: "text",
                schema: c::schema::<MorseStream>,
                required: true,
            },
            c::Port {
                name: "padding_count",
                semantic_type: "json",
                schema: c::schema::<PairPadCount>,
                required: true,
            },
            c::Port {
                name: "padding_ambiguous",
                semantic_type: "json",
                schema: c::schema::<bool>,
                required: true,
            },
        ],
        fixture: fixtures::morbit_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Morbit.pdf"],
    }),
    row(Spec {
        name: "Morbit",
        description: "Morbit: explicit bounded decode; see typed profile guide for conventions.",
        reversible: false,
        output_type: "json",
        id: "cipher.morbit.decode",
        python: "morbit_decode",
        rust: "cube_core::ciphers::fractionation::morse::morbit",
        command: "cipher morbit",
        parameters: c::schema::<morse::Morbit>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<MorbitText>,
            required: true,
        }],
        outputs: &[
            c::Port {
                name: "text",
                semantic_type: "text",
                schema: c::schema::<Option<MorsePlain>>,
                required: true,
            },
            c::Port {
                name: "ciphertext",
                semantic_type: "text",
                schema: c::schema::<MorbitText>,
                required: true,
            },
            c::Port {
                name: "morse",
                semantic_type: "text",
                schema: c::schema::<MorseStream>,
                required: true,
            },
            c::Port {
                name: "padding_count",
                semantic_type: "json",
                schema: c::schema::<PairPadCount>,
                required: true,
            },
            c::Port {
                name: "padding_ambiguous",
                semantic_type: "json",
                schema: c::schema::<bool>,
                required: true,
            },
        ],
        fixture: fixtures::morbit_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Morbit.pdf"],
    }),
    row(Spec {
        name: "Pollux",
        description: "Pollux: explicit bounded encode; see typed profile guide for conventions.",
        reversible: false,
        output_type: "json",
        id: "cipher.pollux.encode",
        python: "pollux_encode",
        rust: "cube_core::ciphers::fractionation::morse::pollux",
        command: "cipher pollux",
        parameters: c::schema::<morse::Pollux>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<MorsePlain>,
            required: true,
        }],
        outputs: &[
            c::Port {
                name: "text",
                semantic_type: "text",
                schema: c::schema::<Option<MorsePlain>>,
                required: true,
            },
            c::Port {
                name: "ciphertext",
                semantic_type: "text",
                schema: c::schema::<PolluxText>,
                required: true,
            },
            c::Port {
                name: "morse",
                semantic_type: "text",
                schema: c::schema::<MorseStream>,
                required: true,
            },
            c::Port {
                name: "padding_count",
                semantic_type: "json",
                schema: c::schema::<NoPadding>,
                required: true,
            },
            c::Port {
                name: "padding_ambiguous",
                semantic_type: "json",
                schema: c::schema::<bool>,
                required: true,
            },
        ],
        fixture: fixtures::pollux_encode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Pollux.pdf"],
    }),
    row(Spec {
        name: "Pollux",
        description: "Pollux: explicit bounded decode; see typed profile guide for conventions.",
        reversible: false,
        output_type: "json",
        id: "cipher.pollux.decode",
        python: "pollux_decode",
        rust: "cube_core::ciphers::fractionation::morse::pollux",
        command: "cipher pollux",
        parameters: c::schema::<morse::Pollux>,
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<PolluxText>,
            required: true,
        }],
        outputs: &[
            c::Port {
                name: "text",
                semantic_type: "text",
                schema: c::schema::<Option<MorsePlain>>,
                required: true,
            },
            c::Port {
                name: "ciphertext",
                semantic_type: "text",
                schema: c::schema::<PolluxText>,
                required: true,
            },
            c::Port {
                name: "morse",
                semantic_type: "text",
                schema: c::schema::<MorseStream>,
                required: true,
            },
            c::Port {
                name: "padding_count",
                semantic_type: "json",
                schema: c::schema::<NoPadding>,
                required: true,
            },
            c::Port {
                name: "padding_ambiguous",
                semantic_type: "json",
                schema: c::schema::<bool>,
                required: true,
            },
        ],
        fixture: fixtures::pollux_decode,
        input_max: 1048576,
        output_max: 2097152,
        sensitive: &[],
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Pollux.pdf"],
    }),
];
const fn profiles() -> [m::Entry; COUNT] {
    let mut out = [ROWS[0].profile; COUNT];
    let mut i = 0;
    while i < COUNT {
        out[i] = ROWS[i].profile;
        i += 1;
    }
    out
}
const fn contracts() -> [c::Entry; COUNT] {
    let mut out = [ROWS[0].contract; COUNT];
    let mut i = 0;
    while i < COUNT {
        out[i] = ROWS[i].contract;
        i += 1;
    }
    out
}
static PROFILES: [m::Entry; COUNT] = profiles();
const CONTRACTS: [c::Entry; COUNT] = contracts();
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILES);
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILES)
    .with_contracts(&CONTRACTS)
    .with_executors(execution::ENTRIES);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_answers_and_complete_typed_contracts() {
        for row in ROWS {
            let fixture = (row.contract.fixture)().unwrap();
            let contract = c::describe(&row.contract.descriptor).unwrap();
            contract
                .validate_fixture(&fixture)
                .unwrap_or_else(|e| panic!("{}: {e}", contract.operation));
            if fixture.inputs.get("text").is_some() {
                assert!(
                    contract
                        .validate_inputs(&serde_json::json!({"text": null}))
                        .is_err(),
                    "{} must require a real text input",
                    contract.operation
                );
            }
            let mut bad = fixture;
            bad.parameters
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), serde_json::json!(true));
            assert!(contract.validate_fixture(&bad).is_err());
        }
    }
}
