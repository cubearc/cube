//! Module-owned descriptor/profile rows; canonical typed contracts use the shared API.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const fn param(
    name: &'static str,
    description: &'static str,
    required: bool,
    default: Option<o::ParameterValue<'static>>,
    choices: &'static [&'static str],
) -> o::Parameter {
    o::Parameter {
        name,
        description,
        required,
        default,
        constraints: o::ParameterType::Text { choices },
    }
}
const SUB_PARAMS: &[o::Parameter] = &[
    param("source", "Exact ordered source alphabet.", true, None, &[]),
    param(
        "target",
        "Explicit target permutation; choose exactly one target or keyword.",
        false,
        None,
        &[],
    ),
    param(
        "keyword",
        "Explicit keyword; choose exactly one target or keyword.",
        false,
        None,
        &[],
    ),
    param(
        "unmapped",
        "Reject unmatched scalars or preserve them exactly; preserve is not always invertible.",
        false,
        Some(o::ParameterValue::Text("reject")),
        &["reject", "preserve"],
    ),
];
const DEFINITION_PARAMS: &[o::Parameter] = &[param(
    "definition",
    "Strict bounded JSON text for the caller-owned table.",
    true,
    None,
    &[],
)];
const HOM_PARAMS: &[o::Parameter] = &[
    param(
        "definition",
        "Strict bounded JSON text for the caller-owned table.",
        true,
        None,
        &[],
    ),
    param(
        "counters",
        "Strict JSON starting indexes (0..65535), empty list by default.",
        false,
        Some(o::ParameterValue::Text("[]")),
        &[],
    ),
];
const fn resource(name: &'static str, maximum: usize) -> m::ResourceBound {
    m::ResourceBound {
        name,
        maximum: maximum as u64,
        unit: "bytes",
        scope: m::Scope::Core,
    }
}
struct Spec {
    inputs: &'static [c::Port],
    parameters: fn() -> schemars::Schema,
    outputs: &'static [c::Port],
    limits: c::Limits,
    fixture: fn() -> Result<c::Fixture, crate::errors::OperationError>,
    descriptor: o::Descriptor,
    name: &'static str,
    rust: &'static str,
    python: &'static str,
    alphabet: &'static str,
    key: &'static str,
    resources: &'static [m::ResourceBound],
    references: &'static [&'static str],
}
#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
}
const fn row(s: Spec) -> Row {
    Row {
        contract: c::Entry {
            descriptor: s.descriptor,
            inputs: s.inputs,
            parameters: s.parameters,
            outputs: s.outputs,
            limits: s.limits,
            fixture: s.fixture,
        },
        profile: m::Entry {
            descriptor: s.descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: s.name,
                family: "monoalphabetic_substitution",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::HistoricalCipher,
                    limitations: "Historical/challenge transformation only; no modern confidentiality, authentication, random selection or certification claim. Canonicalization and preserved-symbol limitations apply as documented.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: s.descriptor.description,
                    alphabet: s.alphabet,
                    key: s.key,
                    key_advancement: "Where applicable, per-symbol round robin resets on each call; no hidden state or random source.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: s.resources,
                resource_notes: "Bounds apply to every direct caller. Token-based operations additionally allow at most 65536 logical tokens. Table definitions are capped at 64 KiB. Conservative Core temporary heap/work maxima derive from these fixed caps, excluding caller-owned and Python/Play object overhead. No pipeline adapter is enabled by metadata.",
                rust_api: s.rust,
                python_api: s.python,
                references: s.references,
            }),
        },
    }
}

const ROWS: [Row; 15] = [
    row(Spec {
        name: "Explicit/keyword substitution",
        rust: "cube_core::ciphers::mono::alphabet::Mapping::encode",
        python: "substitution_encode",
        alphabet: "Exact scalars, including combining marks/NUL; no normalization or folding.",
        key: "Required source and exactly one explicit target or keyword.",
        descriptor: o::Descriptor {
            id: "cipher.substitution.encode",
            category: "ciphers",
            description: "Encode exact Unicode scalars using a validated bijective alphabet; reject by default, <=1 MiB input/4 MiB output.",
            input_type: "text",
            output_type: "text",
            parameters: SUB_PARAMS,
            deterministic: true,
            reversible: Some(true),
            command: "cipher substitution",
            example: "cube cipher substitution --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Substitution>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::ExpandedText>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: None,
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::substitution_encode,
        references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
    }),
    row(Spec {
        name: "Explicit/keyword substitution",
        rust: "cube_core::ciphers::mono::alphabet::Mapping::decode",
        python: "substitution_decode",
        alphabet: "Exact scalars, including combining marks/NUL; no normalization or folding.",
        key: "Required source and exactly one explicit target or keyword.",
        descriptor: o::Descriptor {
            id: "cipher.substitution.decode",
            category: "ciphers",
            description: "Decode exact Unicode scalars using a validated bijective alphabet; reject by default, <=1 MiB input/4 MiB output.",
            input_type: "text",
            output_type: "text",
            parameters: SUB_PARAMS,
            deterministic: true,
            reversible: Some(true),
            command: "cipher substitution",
            example: "cube cipher substitution --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Substitution>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::ExpandedText>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: None,
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::substitution_decode,
        references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
    }),
    row(Spec {
        name: "ROT5",
        rust: "cube_core::ciphers::mono::rotations::rot5",
        python: "rot5",
        alphabet: "Only the named ASCII range rotates; preserve spaces, NUL and Unicode byte-for-byte.",
        key: "No key.",
        descriptor: o::Descriptor {
            id: "cipher.rot5",
            category: "ciphers",
            description: "Self-inverse ASCII digit rotation by five; preserve other scalars, <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "cipher rot5",
            example: "cube cipher rot5 --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 1048576),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Empty>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(1048576),
            max_items: None,
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::rot5,
        references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
    }),
    row(Spec {
        name: "ROT18",
        rust: "cube_core::ciphers::mono::rotations::rot18",
        python: "rot18",
        alphabet: "Only the named ASCII range rotates; preserve spaces, NUL and Unicode byte-for-byte.",
        key: "No key.",
        descriptor: o::Descriptor {
            id: "cipher.rot18",
            category: "ciphers",
            description: "Self-inverse ASCII ROT13 letters and ROT5 digits; preserve other scalars, <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "cipher rot18",
            example: "cube cipher rot18 --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 1048576),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Empty>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(1048576),
            max_items: None,
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::rot18,
        references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
    }),
    row(Spec {
        name: "ROT47",
        rust: "cube_core::ciphers::mono::rotations::rot47",
        python: "rot47",
        alphabet: "Only the named ASCII range rotates; preserve spaces, NUL and Unicode byte-for-byte.",
        key: "No key.",
        descriptor: o::Descriptor {
            id: "cipher.rot47",
            category: "ciphers",
            description: "Self-inverse printable ASCII U+0021..U+007E rotation by 47; preserve other scalars, <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "cipher rot47",
            example: "cube cipher rot47 --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 1048576),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Empty>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(1048576),
            max_items: None,
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::rot47,
        references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
    }),
    row(Spec {
        name: "Deterministic homophonic substitution",
        rust: "cube_core::ciphers::mono::homophonic::Mapping::encode",
        python: "homophonic_encode",
        alphabet: "Exact source scalars, or explicit ASCII-uppercase canonicalization; tokens are UTF-8 strings, never concatenated.",
        key: "Caller-owned table; each source 1..256 tokens, global ownership unique.",
        descriptor: o::Descriptor {
            id: "cipher.homophonic.encode",
            category: "ciphers",
            description: "Encode canonical scalars to explicit tokens using local per-symbol round robin.",
            input_type: "text",
            output_type: "cipher.symbol_tokens",
            parameters: HOM_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher homophonic",
            example: "cube cipher homophonic --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 1048576),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::HomophonicEncode>,
        outputs: &[c::Port {
            name: "tokens",
            semantic_type: "cipher.symbol_tokens",
            schema: c::schema::<super::params::Tokens>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(1048576),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::homophonic_encode,
        references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
    }),
    row(Spec {
        name: "Deterministic homophonic substitution",
        rust: "cube_core::ciphers::mono::homophonic::Mapping::decode",
        python: "homophonic_decode",
        alphabet: "Exact source scalars, or explicit ASCII-uppercase canonicalization; tokens are UTF-8 strings, never concatenated.",
        key: "Caller-owned table; each source 1..256 tokens, global ownership unique.",
        descriptor: o::Descriptor {
            id: "cipher.homophonic.decode",
            category: "ciphers",
            description: "Decode exact owned tokens to canonical text.",
            input_type: "cipher.symbol_tokens",
            output_type: "text",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher homophonic",
            example: "cube cipher homophonic --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 1048576),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "tokens",
            semantic_type: "cipher.symbol_tokens",
            schema: c::schema::<super::params::Tokens>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::homophonic::Definition>>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(1048576),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::homophonic_decode,
        references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
    }),
    row(Spec {
        name: "Straddling checkerboard",
        rust: "cube_core::ciphers::mono::checkerboard::Table::encode",
        python: "straddling_encode",
        alphabet: "ASCII letters uppercase; spaces require a cell except Tridigital reserves the tenth digit. Strict ungrouped digit transport.",
        key: "Explicit digit permutation, prefixes, ordered alphabet and aliases; optional Straddling-only digit escape.",
        descriptor: o::Descriptor {
            id: "cipher.straddling.encode",
            category: "ciphers",
            description: "Encode with an explicit straddling digit table; reject unassigned/incomplete digit transport.",
            input_type: "text",
            output_type: "text",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher straddling",
            example: "cube cipher straddling --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::checkerboard::Definition>>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::ExpandedText>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::straddling_encode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Checkerboard.pdf"],
    }),
    row(Spec {
        name: "Straddling checkerboard",
        rust: "cube_core::ciphers::mono::checkerboard::Table::decode",
        python: "straddling_decode",
        alphabet: "ASCII letters uppercase; spaces require a cell except Tridigital reserves the tenth digit. Strict ungrouped digit transport.",
        key: "Explicit digit permutation, prefixes, ordered alphabet and aliases; optional Straddling-only digit escape.",
        descriptor: o::Descriptor {
            id: "cipher.straddling.decode",
            category: "ciphers",
            description: "Decode with an explicit straddling digit table; reject unassigned/incomplete digit transport.",
            input_type: "text",
            output_type: "text",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher straddling",
            example: "cube cipher straddling --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::checkerboard::Definition>>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::ExpandedText>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::straddling_decode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Checkerboard.pdf"],
    }),
    row(Spec {
        name: "ACA Monome-Dinome",
        rust: "cube_core::ciphers::mono::checkerboard::Table::encode",
        python: "monome_dinome_encode",
        alphabet: "ASCII letters uppercase; spaces require a cell except Tridigital reserves the tenth digit. Strict ungrouped digit transport.",
        key: "Explicit digit permutation, prefixes, ordered alphabet and aliases; optional Straddling-only digit escape.",
        descriptor: o::Descriptor {
            id: "cipher.monome_dinome.encode",
            category: "ciphers",
            description: "Encode with an explicit monome_dinome digit table; reject unassigned/incomplete digit transport.",
            input_type: "text",
            output_type: "text",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher monome-dinome",
            example: "cube cipher monome-dinome --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::checkerboard::Definition>>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::ExpandedText>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::monome_dinome_encode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/MonomeDinome.pdf"],
    }),
    row(Spec {
        name: "ACA Monome-Dinome",
        rust: "cube_core::ciphers::mono::checkerboard::Table::decode",
        python: "monome_dinome_decode",
        alphabet: "ASCII letters uppercase; spaces require a cell except Tridigital reserves the tenth digit. Strict ungrouped digit transport.",
        key: "Explicit digit permutation, prefixes, ordered alphabet and aliases; optional Straddling-only digit escape.",
        descriptor: o::Descriptor {
            id: "cipher.monome_dinome.decode",
            category: "ciphers",
            description: "Decode with an explicit monome_dinome digit table; reject unassigned/incomplete digit transport.",
            input_type: "text",
            output_type: "text",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher monome-dinome",
            example: "cube cipher monome-dinome --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::checkerboard::Definition>>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::ExpandedText>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::monome_dinome_decode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/MonomeDinome.pdf"],
    }),
    row(Spec {
        name: "ACA Tridigital",
        rust: "cube_core::ciphers::mono::checkerboard::Table::encode",
        python: "tridigital_encode",
        alphabet: "ASCII letters uppercase; spaces require a cell except Tridigital reserves the tenth digit. Strict ungrouped digit transport.",
        key: "Explicit digit permutation, prefixes, ordered alphabet and aliases; optional Straddling-only digit escape.",
        descriptor: o::Descriptor {
            id: "cipher.tridigital.encode",
            category: "ciphers",
            description: "Encode with an explicit tridigital digit table; reject unassigned/incomplete digit transport.",
            input_type: "text",
            output_type: "text",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher tridigital",
            example: "cube cipher tridigital --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::checkerboard::Definition>>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::ExpandedText>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::tridigital_encode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Tridigital.pdf"],
    }),
    row(Spec {
        name: "ACA Tridigital",
        rust: "cube_core::ciphers::mono::checkerboard::Table::alternatives",
        python: "tridigital_decode",
        alphabet: "ASCII letters uppercase; spaces require a cell except Tridigital reserves the tenth digit. Strict ungrouped digit transport.",
        key: "Explicit digit permutation, prefixes, ordered alphabet and aliases; optional Straddling-only digit escape.",
        descriptor: o::Descriptor {
            id: "cipher.tridigital.decode",
            category: "ciphers",
            description: "Decode with an explicit tridigital digit table; return per-digit alternatives without guessing.",
            input_type: "text",
            output_type: "cipher.alternatives",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher tridigital",
            example: "cube cipher tridigital --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 4194304),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::checkerboard::Definition>>,
        outputs: &[c::Port {
            name: "alternatives",
            semantic_type: "cipher.alternatives",
            schema: c::schema::<super::params::Alternatives>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(4194304),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::tridigital_decode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Tridigital.pdf"],
    }),
    row(Spec {
        name: "Grandpré",
        rust: "cube_core::ciphers::mono::grandpre::Square::encode",
        python: "grandpre_encode",
        alphabet: "ASCII uppercase canonical letters; explicit null space tokens. Repeated grid letters are legal.",
        key: "Caller-supplied 8x8, 9x9 or 10x10 letter rows, explicit row/column digit permutations.",
        descriptor: o::Descriptor {
            id: "cipher.grandpre.encode",
            category: "ciphers",
            description: "Encode repeated-letter 8/9/10-square coordinates with explicit digit labels; null represents each space.",
            input_type: "text",
            output_type: "cipher.coordinate_tokens",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher grandpre",
            example: "cube cipher grandpre --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 1048576),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::grandpre::Definition>>,
        outputs: &[c::Port {
            name: "tokens",
            semantic_type: "cipher.coordinate_tokens",
            schema: c::schema::<super::params::Coordinates>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(1048576),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::grandpre_encode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Grandpre.pdf"],
    }),
    row(Spec {
        name: "Grandpré",
        rust: "cube_core::ciphers::mono::grandpre::Square::decode",
        python: "grandpre_decode",
        alphabet: "ASCII uppercase canonical letters; explicit null space tokens. Repeated grid letters are legal.",
        key: "Caller-supplied 8x8, 9x9 or 10x10 letter rows, explicit row/column digit permutations.",
        descriptor: o::Descriptor {
            id: "cipher.grandpre.decode",
            category: "ciphers",
            description: "Decode repeated-letter 8/9/10-square coordinates with explicit digit labels; null represents each space.",
            input_type: "cipher.coordinate_tokens",
            output_type: "text",
            parameters: DEFINITION_PARAMS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher grandpre",
            example: "cube cipher grandpre --help",
        },
        resources: &[
            resource("input_utf8_bytes", MAX_INPUT),
            resource("output_utf8_bytes", 1048576),
            resource("definition_utf8_bytes", MAX_DEFINITION),
        ],
        inputs: &[c::Port {
            name: "tokens",
            semantic_type: "cipher.coordinate_tokens",
            schema: c::schema::<super::params::Coordinates>,
            required: true,
        }],
        parameters: c::schema::<super::params::Definition<super::grandpre::Definition>>,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::params::Text>,
            required: true,
        }],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(1048576),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: super::fixtures::grandpre_decode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Grandpre.pdf"],
    }),
];
const fn profiles() -> [m::Entry; 15] {
    let mut a = [ROWS[0].profile; 15];
    let mut i = 0;
    while i < 15 {
        a[i] = ROWS[i].profile;
        i += 1;
    }
    a
}
const fn contracts() -> [c::Entry; 15] {
    let mut a = [ROWS[0].contract; 15];
    let mut i = 0;
    while i < 15 {
        a[i] = ROWS[i].contract;
        i += 1;
    }
    a
}
const PROFILED: [m::Entry; 15] = profiles();
const CONTRACTS: [c::Entry; 15] = contracts();
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(&CONTRACTS);

#[cfg(test)]
mod tests {
    use super::*;
    fn has_serialized_budget(value: &serde_json::Value, maximum: usize) -> bool {
        match value {
            serde_json::Value::Object(fields) => {
                fields.get("x-cube-max-serialized-bytes") == Some(&serde_json::json!(maximum))
                    || fields.values().any(|v| has_serialized_budget(v, maximum))
            }
            serde_json::Value::Array(items) => {
                items.iter().any(|v| has_serialized_budget(v, maximum))
            }
            _ => false,
        }
    }
    #[test]
    fn every_exported_cipher_has_real_conforming_known_answers_and_rejects_bad_shapes() {
        let mut ids = std::collections::BTreeSet::new();
        for row in ROWS {
            let descriptor = &row.profile.descriptor;
            assert!(ids.insert(descriptor.id));
            let contract = c::describe(descriptor).unwrap();
            let fixture = (row.contract.fixture)().unwrap();
            contract
                .validate_fixture(&fixture)
                .unwrap_or_else(|e| panic!("{}: {e}", descriptor.id));
            assert_eq!(contract.operation, descriptor.id);
            assert_eq!(contract.operation_version, 1);
            assert_eq!(contract.inputs.len(), 1);
            assert_eq!(contract.outputs.len(), 1);
            if descriptor.parameters.iter().any(|p| p.name == "definition") {
                assert!(has_serialized_budget(
                    contract.parameters.as_value(),
                    MAX_DEFINITION
                ));
            }
            if descriptor.parameters.iter().any(|p| p.name == "source") {
                assert_eq!(
                    contract.parameters.as_value()["x-cube-max-combined-alphabet-utf8-bytes"],
                    serde_json::json!(MAX_DEFINITION)
                );
            }
            assert!(
                !contract.policy.grants_filesystem_access && !contract.policy.grants_network_access
            );
            let mut invalid = (row.contract.fixture)().unwrap();
            invalid
                .parameters
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), serde_json::json!(true));
            assert!(
                contract.validate_fixture(&invalid).is_err(),
                "{} unknown parameters",
                descriptor.id
            );
            let mut invalid = (row.contract.fixture)().unwrap();
            invalid
                .inputs
                .as_object_mut()
                .unwrap()
                .insert(contract.inputs[0].name.clone(), serde_json::json!(true));
            assert!(
                contract.validate_fixture(&invalid).is_err(),
                "{} input type",
                descriptor.id
            );
            let mut invalid = (row.contract.fixture)().unwrap();
            invalid.outputs = serde_json::json!({});
            assert!(
                contract.validate_fixture(&invalid).is_err(),
                "{} missing named output",
                descriptor.id
            );
        }
        assert_eq!(ids, OPERATIONS.iter().map(|d| d.id).collect());
    }
}
