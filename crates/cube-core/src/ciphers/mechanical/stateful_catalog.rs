//! Canonical direct adapters for independently owned historical mechanisms.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const fn outputs<R: schemars::JsonSchema, S: schemars::JsonSchema>() -> [c::Port; 3] {
    [
        c::Port {
            name: "text",
            semantic_type: "text",
            schema: || c::field::<R>("text"),
            required: true,
        },
        c::Port {
            name: "final_state",
            semantic_type: "json",
            schema: c::schema::<S>,
            required: true,
        },
        c::Port {
            name: "trace",
            semantic_type: "json",
            schema: || c::field::<R>("trace"),
            required: true,
        },
    ]
}
const WHEATSTONE_OUTPUTS: &[c::Port] = &outputs::<wheatstone::Report, wheatstone::State>();
const KRYHA_OUTPUTS: &[c::Port] = &outputs::<kryha::Report, kryha::State>();
const ALBERTI_OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<alberti::Report>("text"),
        required: true,
    },
    c::Port {
        name: "final_index",
        semantic_type: "text",
        schema: || c::field::<alberti::Report>("final_index"),
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<alberti::Report>("trace"),
        required: true,
    },
];
const fn inputs(settings: fn() -> schemars::Schema) -> [c::Port; 2] {
    [
        c::Port {
            name: "text",
            semantic_type: "text",
            schema: || c::field::<disks::Report>("text"),
            required: true,
        },
        c::Port {
            name: "settings",
            semantic_type: "json",
            schema: settings,
            required: true,
        },
    ]
}
struct Family {
    variant: &'static str,
    alphabet: &'static str,
    key: &'static str,
    advancement: &'static str,
    rust: &'static str,
    references: &'static [&'static str],
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
}
const WHEATSTONE: Family = Family {
    variant: "Wheatstone 1879 original 27/26-cell clock cryptograph, with exact prepared text and explicit initial pointers.",
    alphabet: "Fixed +ABCDEFGHIJKLMNOPQRSTUVWXYZ outer ring; caller-supplied mixed A-Z cipher ring. No repeated prepared plain symbols or automatic filler.",
    key: "Required cipher alphabet and initial 27/26-cell pointer positions. Optional trace defaults to false.",
    advancement: "Strictly positive clockwise plain motion 1..26 advances the inner pointer equally; decode treats equal cipher symbols as 26 advances.",
    rust: "cube_core::ciphers::mechanical::wheatstone",
    references: &[
        "https://incoherency.co.uk/interest/several-machine-ciphers.pdf",
        "https://www.jproc.ca/crypto/wheatstone.html",
    ],
    inputs: &inputs(c::schema::<wheatstone::Settings>),
    outputs: WHEATSTONE_OUTPUTS,
};
const KRYHA: Family = Family {
    variant: "Kryha 17-sector source-specific profile from Callimahos 1973 Q.E.D., effective displacement table on pp. 20–21.",
    alphabet: "Required plain/cipher permutations of exact A-Z. No word separation or text normalization.",
    key: "Required two mixed alphabets, initial rotation 0..25 and numbered selector 1..17. Optional trace defaults to false.",
    advancement: "Use the current alignment, then advance by the fixed 17-entry effective gear schedule; initial selector chooses the next post-letter step.",
    rust: "cube_core::ciphers::mechanical::kryha",
    references: &[
        "https://www.nsa.gov/portals/75/documents/news-features/declassified-documents/tech-journals/qed.pdf",
        "https://cryptocellar.org/kryha/index.html",
    ],
    inputs: &inputs(c::schema::<kryha::Settings>),
    outputs: KRYHA_OUTPUTS,
};
const ALBERTI: Family = Family {
    variant: "Alberti digit-feedback profile from Di Palma 2009 museum reconstruction, exact 24-cell rings and explicit starting index.",
    alphabet: "Outer ABCDEFGILMNOPQRSTVXZ1234; inner c&bmdgpfznxyutoskerlhaiq. Canonical uppercase V/lowercase u; no automatic U/V folding or preparation.",
    key: "Required initial inner symbol under fixed A; digit indicators are caller-supplied prepared data and retained on decode. Optional trace defaults to false.",
    advancement: "After each enciphered or recovered digit 1..4, realign the emitted inner symbol under fixed A; all other letters preserve alignment.",
    rust: "cube_core::ciphers::mechanical::alberti",
    references: &["https://www2.mathematik.uni-halle.de/reports/sources/2009/09-04report.pdf"],
    inputs: &inputs(c::schema::<alberti::Settings>),
    outputs: ALBERTI_OUTPUTS,
};
#[derive(Clone, Copy)]
struct Spec {
    id: &'static str,
    name: &'static str,
    command: &'static str,
    python: &'static str,
    family: &'static Family,
    run: fn(&str, &Value) -> Result<Value, OperationError>,
    fixture: fn() -> Result<c::Fixture, OperationError>,
}
const fn entry(s: Spec) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id: s.id,
            category: "ciphers",
            description: s.family.variant,
            input_type: "json",
            output_type: "json",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: s.command,
            example: "cube cipher --help",
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: s.name,
            family: "historical_mechanical",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Historical interoperability only; no modern confidentiality or authentication. Settings, plaintext and state traces are private.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: s.family.variant,
                alphabet: s.family.alphabet,
                key: s.family.key,
                key_advancement: s.family.advancement,
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text <=1 MiB; settings <=512 KiB; optional trace <=65536 symbols; complete report <=4 MiB, failing instead of truncating. Bounded fixed wheel work per symbol. Pure/local execution; no implicit capture or linear recipe adapter.",
            rust_api: s.family.rust,
            python_api: s.python,
            references: s.family.references,
        }),
    }
}
const ROWS: [Spec; 6] = [
    Spec {
        id: "cipher.wheatstone.encode",
        name: "Encipher with Wheatstone's cryptograph",
        command: "cipher wheatstone encode",
        python: "wheatstone_encode",
        family: &WHEATSTONE,
        run: wheatstone_encode,
        fixture: wheatstone_encode_fixture,
    },
    Spec {
        id: "cipher.wheatstone.decode",
        name: "Decipher with Wheatstone's cryptograph",
        command: "cipher wheatstone decode",
        python: "wheatstone_decode",
        family: &WHEATSTONE,
        run: wheatstone_decode,
        fixture: wheatstone_decode_fixture,
    },
    Spec {
        id: "cipher.kryha.encode",
        name: "Encipher with Kryha's 17-sector profile",
        command: "cipher kryha encode",
        python: "kryha_encode",
        family: &KRYHA,
        run: kryha_encode,
        fixture: kryha_encode_fixture,
    },
    Spec {
        id: "cipher.kryha.decode",
        name: "Decipher with Kryha's 17-sector profile",
        command: "cipher kryha decode",
        python: "kryha_decode",
        family: &KRYHA,
        run: kryha_decode,
        fixture: kryha_decode_fixture,
    },
    Spec {
        id: "cipher.alberti.encode",
        name: "Encipher with Alberti's digit-feedback disk",
        command: "cipher alberti encode",
        python: "alberti_encode",
        family: &ALBERTI,
        run: alberti_encode,
        fixture: alberti_encode_fixture,
    },
    Spec {
        id: "cipher.alberti.decode",
        name: "Decipher with Alberti's digit-feedback disk",
        command: "cipher alberti decode",
        python: "alberti_decode",
        family: &ALBERTI,
        run: alberti_decode,
        fixture: alberti_decode_fixture,
    },
];
const PROFILED: [m::Entry; 6] = [
    entry(ROWS[0]),
    entry(ROWS[1]),
    entry(ROWS[2]),
    entry(ROWS[3]),
    entry(ROWS[4]),
    entry(ROWS[5]),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const fn contract(s: Spec) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: entry(s).descriptor,
        inputs: s.family.inputs,
        parameters: c::schema::<super::catalog::NoParameters>,
        outputs: s.family.outputs,
        limits: c::Limits {
            max_input_bytes: Some((MAX_INPUT + MAX_PARAMETERS) as u64),
            max_output_bytes: Some(MAX_OUTPUT as u64),
            max_items: Some(MAX_TRACE as u64),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture: s.fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[
    contract(ROWS[0]),
    contract(ROWS[1]),
    contract(ROWS[2]),
    contract(ROWS[3]),
    contract(ROWS[4]),
    contract(ROWS[5]),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
pub(super) fn execute(
    op: &str,
    inputs: &Value,
    _parameters: &Value,
) -> Result<Value, OperationError> {
    let s = ROWS.iter().find(|s| s.id == op).ok_or_else(|| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "unknown stateful mechanical operation",
        )
    })?;
    let text = inputs
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| error(s.id, ErrorCode::InvalidInput, "text", "expected text"))?;
    let settings = inputs.get("settings").ok_or_else(|| {
        error(
            s.id,
            ErrorCode::InvalidInput,
            "settings",
            "explicit machine settings required",
        )
    })?;
    (s.run)(text, settings)
}
fn native<P: serde::de::DeserializeOwned, R: Serialize>(
    op: &'static str,
    text: &str,
    settings: &Value,
    run: fn(&str, &P) -> Result<R, OperationError>,
) -> Result<Value, OperationError> {
    let p = parse_parameters(&settings.to_string(), op)?;
    serde_json::to_value(run(text, &p)?).map_err(|_| {
        error(
            op,
            ErrorCode::InvalidInput,
            "report",
            "typed report serialization failed",
        )
    })
}
fn wheatstone_encode(t: &str, s: &Value) -> Result<Value, OperationError> {
    native("cipher.wheatstone.encode", t, s, wheatstone::encode)
}
fn wheatstone_decode(t: &str, s: &Value) -> Result<Value, OperationError> {
    native("cipher.wheatstone.decode", t, s, wheatstone::decode)
}
fn kryha_encode(t: &str, s: &Value) -> Result<Value, OperationError> {
    native("cipher.kryha.encode", t, s, kryha::encode)
}
fn kryha_decode(t: &str, s: &Value) -> Result<Value, OperationError> {
    native("cipher.kryha.decode", t, s, kryha::decode)
}
fn alberti_encode(t: &str, s: &Value) -> Result<Value, OperationError> {
    native("cipher.alberti.encode", t, s, alberti::encode)
}
fn alberti_decode(t: &str, s: &Value) -> Result<Value, OperationError> {
    native("cipher.alberti.decode", t, s, alberti::decode)
}
fn fixture(
    index: usize,
    settings: Value,
    plain: &str,
    cipher: &str,
) -> Result<c::Fixture, OperationError> {
    let (text, expected) = if index.is_multiple_of(2) {
        (plain, cipher)
    } else {
        (cipher, plain)
    };
    let inputs = json!({"text":text,"settings":settings});
    let outputs = execute(ROWS[index].id, &inputs, &json!({}))?;
    assert_eq!(outputs["text"], expected);
    Ok(c::Fixture {
        inputs,
        parameters: json!({}),
        outputs,
    })
}
fn wheatstone_fixture(index: usize) -> Result<c::Fixture, OperationError> {
    fixture(
        index,
        json!({"profile":"wheatstone_1879_v1","cipher_alphabet":"FBKSYRDLTZAGMUNHOVCIPWEJQX","initial_state":{"plain_position":0,"cipher_position":0},"trace":true}),
        "THE+QUARTER+",
        "PZLSPQREQAJD",
    )
}
fn wheatstone_encode_fixture() -> Result<c::Fixture, OperationError> {
    wheatstone_fixture(0)
}
fn wheatstone_decode_fixture() -> Result<c::Fixture, OperationError> {
    wheatstone_fixture(1)
}
fn kryha_fixture(index: usize) -> Result<c::Fixture, OperationError> {
    fixture(
        index,
        json!({"profile":"kryha_callimahos_1973_v1","plain_alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","cipher_alphabet":"HYDRAULICBEFGJKMNOPQSTVWXZ","initial_state":{"rotation":0,"selector":1},"trace":true}),
        "AAAAAAAAAAAAAAAAA",
        "HIJSZUGPHLNTYLJQX",
    )
}
fn kryha_encode_fixture() -> Result<c::Fixture, OperationError> {
    kryha_fixture(2)
}
fn kryha_decode_fixture() -> Result<c::Fixture, OperationError> {
    kryha_fixture(3)
}
fn alberti_fixture(index: usize) -> Result<c::Fixture, OperationError> {
    fixture(
        index,
        json!({"profile":"alberti_di_palma_2009_v1","initial_index":"c","trace":true}),
        "ALE4ISA1BEAVTI3FVL2CITI",
        "czdqpoqlhqluybeix&okapa",
    )
}
fn alberti_encode_fixture() -> Result<c::Fixture, OperationError> {
    alberti_fixture(4)
}
fn alberti_decode_fixture() -> Result<c::Fixture, OperationError> {
    alberti_fixture(5)
}
