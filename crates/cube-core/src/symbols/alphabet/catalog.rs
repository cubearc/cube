use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const UNITS: &[c::Port] = &[c::Port {
    name: "units",
    semantic_type: "alphabet.units",
    schema: c::schema::<h::Tokens>,
    required: true,
}];
const TOKENS: &[c::Port] = &[c::Port {
    name: "tokens",
    semantic_type: "alphabet.tokens",
    schema: c::schema::<h::Tokens>,
    required: true,
}];
const PROVENANCE: c::Port = c::Port {
    name: "provenance",
    semantic_type: "alphabet.provenance",
    schema: c::schema::<Provenance>,
    required: true,
};
const ENCODE_OUTPUTS: &[c::Port] = &[TOKENS[0], PROVENANCE];
const DECODE_OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "readings",
        semantic_type: "alphabet.readings",
        schema: || c::field::<Decoded>("readings"),
        required: true,
    },
    TOKENS[0],
    PROVENANCE,
];
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "Required dated source profile. Units are exact and presegmented; no inferred font, case, language, digraph or reading order.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text { choices: &[] },
}];
#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
}
const fn row(
    id: &'static str,
    python: &'static str,
    decode: bool,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> Row {
    let descriptor = o::Descriptor {
        id,
        category: "symbols",
        description: if decode {
            "Decode a source-qualified symbolic alphabet, retaining all readings, exact tokens and positions."
        } else {
            "Encode exact presegmented units with an explicit source-qualified symbolic alphabet."
        },
        input_type: if decode {
            "alphabet.tokens"
        } else {
            "alphabet.units"
        },
        output_type: if decode {
            "alphabet.readings"
        } else {
            "alphabet.tokens"
        },
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: if decode {
            "decode alphabet"
        } else {
            "encode alphabet"
        },
        example: "cube encode alphabet --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: if decode { TOKENS } else { UNITS },
            parameters: c::schema::<Parameters>,
            outputs: if decode {
                DECODE_OUTPUTS
            } else {
                ENCODE_OUTPUTS
            },
            limits: c::Limits {
                max_input_bytes: Some(1048576),
                max_output_bytes: Some(MAX_REPORT as u64),
                max_items: Some(65536),
                max_work: None,
                max_memory_bytes: None,
            },
            fixture,
        },
        profile: m::Entry {
            descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: if decode {
                    "Decode a symbolic alphabet"
                } else {
                    "Encode a symbolic alphabet"
                },
                family: "symbolic_alphabet",
                aliases: &[
                    m::Alias {
                        name: "Al Bhed (English FFX)",
                        usage: "profile=al_bhed_english_ffx_v1",
                    },
                    m::Alias {
                        name: "Hylian (Skyward Sword)",
                        usage: "profile=hylian_skyward_dcode_v1",
                    },
                    m::Alias {
                        name: "Sheikah (BOTW reference)",
                        usage: "profile=sheikah_botw_dcode_v1",
                    },
                    m::Alias {
                        name: "Daedric (named letters)",
                        usage: "profile=daedric_names_dcode_v1",
                    },
                    m::Alias {
                        name: "Futurama Alien 1",
                        usage: "profile=futurama_alien1_dcode_v1",
                    },
                    m::Alias {
                        name: "Gravity Falls Bill Cipher (dCode table)",
                        usage: "profile=bill_cipher_dcode_v1",
                    },
                    m::Alias {
                        name: "Gnommish (Disney 2020 activity)",
                        usage: "profile=gnommish_disney_2020_v1",
                    },
                    m::Alias {
                        name: "Theban (Trithemius 1518)",
                        usage: "profile=theban_trithemius_1518_v1",
                    },
                    m::Alias {
                        name: "Celestial",
                        usage: "profile=celestial_agrippa_ch30_v1",
                    },
                    m::Alias {
                        name: "Malachim",
                        usage: "profile=malachim_agrippa_ch30_v1",
                    },
                    m::Alias {
                        name: "Alphabet of the Magi",
                        usage: "profile=magi_ambrosius_1539_v1",
                    },
                    m::Alias {
                        name: "Enochian (Peterson 2022)",
                        usage: "profile=enochian_peterson_2022_v1",
                    },
                    m::Alias {
                        name: "Moon (English 1877)",
                        usage: "profile=moon_english_1877_v1",
                    },
                    m::Alias {
                        name: "Dancing Men (Doyle 1905)",
                        usage: "profile=dancing_men_doyle_1905_v1",
                    },
                    m::Alias {
                        name: "Alphabetum Kaldeorum",
                        usage: "profile=alphabetum_kaldeorum_hahn_2004_v1",
                    },
                    m::Alias {
                        name: "Pigpen",
                        usage: "profile=pigpen_haverhill_pairs_v1",
                    },
                    m::Alias {
                        name: "Rosicrucian (Haverhill paired variant)",
                        usage: "profile=pigpen_haverhill_pairs_v1",
                    },
                    m::Alias {
                        name: "Tic-Tac-Toe",
                        usage: "profile=tic_tac_toe_dcode_v1",
                    },
                    m::Alias {
                        name: "Templars",
                        usage: "profile=templars_bothwell_gosse_v1",
                    },
                    m::Alias {
                        name: "Betamaze",
                        usage: "profile=betamaze_cliff_2006_v1",
                    },
                    m::Alias {
                        name: "Clock quarter-hour code",
                        usage: "profile=clock_quarter_hour_zero_v1",
                    },
                    m::Alias {
                        name: "Birds on a Wire (NR Tech Studio)",
                        usage: "profile=birds_nrtech_four_features_v1",
                    },
                    m::Alias {
                        name: "Aurebesh",
                        usage: "profile=aurebesh_ucsur_2016_v1",
                    },
                    m::Alias {
                        name: "Standard Galactic Alphabet",
                        usage: "profile=standard_galactic_ucsur_2019_v1",
                    },
                    m::Alias {
                        name: "SGA",
                        usage: "profile=standard_galactic_ucsur_2019_v1",
                    },
                ],
                security: m::Security {
                    classification: m::Classification::PuzzleCode,
                    limitations: "Fixed dated alphabet profiles, without confidentiality, OCR, font recognition, language translation or a guessed inverse.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: descriptor.description,
                    alphabet: "Exact source-listed Latin/Hebrew units, semantic marks, explicit digraphs and word-end units. Tokens identify cited chart glyphs, source geometry, bird features/codes, quarter-hour times or published private-use proposals; neutral chart IDs do not invent Unicode encodings. No case, normalization, font or language inference.",
                    key: "Fixed independently transcribed dated source tables, with source and effective-definition hashes. No fonts, source artwork or language translations bundled.",
                    key_advancement: "Canonical first source form per exact unit. Decode preserves original glyph/flag/style tokens and every explicit reading, including Enochian I/J,U/V and Kaldeorum U/V. Betamaze accepts its148orientations; chart aliases retain source variants. No language, orthography, page-direction, maze or word-boundary guessing.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Input <=65536 tokens/1MiB compact JSON; each token <=256 UTF-8 bytes; fixed definition <=64KiB; complete report <=1MiB, including provenance. Only explicit units and aliases are accepted: no automatic segmentation, case changes, marker stripping or unknown-glyph replacement. Missing cells/reserved positions reject. Either direction may reach the report limit before the item limit.",
                rust_api: "cube_core::symbols::alphabet::{encode,decode}",
                python_api: python,
                references: &[
                    "https://www.dcode.fr/al-bhed-language-final-fantasy",
                    "https://www.dcode.fr/hylian-language-skyward-sword",
                    "https://www.dcode.fr/sheikah-language",
                    "https://www.dcode.fr/daedric-language",
                    "https://www.dcode.fr/futurama-alien-alphabet",
                    "https://www.dcode.fr/gravity-falls-bill-cipher",
                    "https://books.disney.com/content/uploads/2020/02/2484_1.pdf",
                    "https://esotericarchives.com/tritheim/polygraphia/poly579.jpg",
                    "https://esotericarchives.com/agrippa/agripp3c.htm#chap30",
                    "https://esotericarchives.com/fonts/index.html",
                    "https://www.unicode.org/wg2/docs/n4128.pdf",
                    "https://www.gutenberg.org/files/108/108-h/108-h.htm#chap03",
                    "https://www.arthur-conan-doyle.com/wiki/Dancing_Men_Alphabet",
                    "https://commons.wikimedia.org/wiki/File:Alphabetum_Kaldeorum.jpg",
                    "https://haverhill-tc.gov.uk/wp-content/uploads/Pigpen-Code.pdf",
                    "https://www.dcode.fr/tic-tac-toe-cipher",
                    "https://freemasonry.bcy.ca/texts/templars_cipher.html",
                    "https://library.conlang.org/education/DECAL_%20reader_part2.pdf",
                    "https://www.dcode.fr/clock-cipher",
                    "https://nrtechstudio.com/tools/birds-on-a-wire-cipher/",
                    "https://www.kreativekorp.com/ucsur/charts/aurebesh.htm",
                    "https://www.kreativekorp.com/ucsur/charts/sga.html",
                ],
            }),
        },
    }
}
const ROWS: [Row; 2] = [
    row(
        "symbol.alphabet.encode",
        "alphabet_encode",
        false,
        super::fixtures::encode,
    ),
    row(
        "symbol.alphabet.decode",
        "alphabet_decode",
        true,
        super::fixtures::decode,
    ),
];
const PROFILED: [m::Entry; 2] = [ROWS[0].profile, ROWS[1].profile];
const CONTRACTS: [c::Entry; 2] = [ROWS[0].contract, ROWS[1].contract];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn native(op: &'static str, i: &Value, p: &Value) -> e::Result<Value> {
    let run = || {
        let p: Parameters = crate::ciphers::poly::parse_parameters(&p.to_string())?;
        let field = if op.ends_with(".decode") {
            "tokens"
        } else {
            "units"
        };
        if i.as_object().is_none_or(|fields| fields.len() != 1) {
            return Err(crate::errors::OperationError::new(
                crate::errors::ErrorCode::InvalidInput,
                "one exact named unit-token input required",
            ));
        }
        let input =
            serde_json::from_value::<Vec<String>>(i.get(field).cloned().unwrap_or(Value::Null))
                .map_err(|_| {
                    crate::errors::OperationError::new(
                        crate::errors::ErrorCode::InvalidInput,
                        "expected exact presegmented unit strings",
                    )
                })?;
        if op.ends_with(".decode") {
            serde_json::to_value(decode(&input, &p)?)
        } else {
            serde_json::to_value(encode(&input, &p)?)
        }
        .map_err(|_| {
            crate::errors::OperationError::new(
                crate::errors::ErrorCode::InvalidInput,
                "typed report serialization failed",
            )
        })
    };
    run().map_err(|e: OperationError| e::Failure(e.code.as_str()))
}
const EXECUTORS: &[e::Entry] = &[
    e::Entry::pure(
        "symbol.alphabet.encode",
        &[e::Representation::Json],
        &[e::Representation::Json, e::Representation::Json],
        |i, p, _| native("symbol.alphabet.encode", i, p),
    ),
    e::Entry::pure(
        "symbol.alphabet.decode",
        &[e::Representation::Json],
        &[
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Json,
        ],
        |i, p, _| native("symbol.alphabet.decode", i, p),
    ),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
