//! Module-owned typed schemas, metadata and independently checked fixtures.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
}
struct Spec {
    id: &'static str,
    name: &'static str,
    command: &'static str,
    python: &'static str,
    rust: &'static str,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    fixture: fn() -> Result<c::Fixture, OperationError>,
}
const fn row(s: Spec) -> Row {
    let descriptor = o::Descriptor {
        id: s.id,
        category: "cryptanalysis",
        description: s.name,
        input_type: "json",
        output_type: "json",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: s.command,
        example: "cube cryptanalysis --help",
    };
    Row {
        profile: m::Entry {
            descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: s.name,
                family: "classical-analysis",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::HistoricalCipher,
                    limitations: "Classical/challenge evidence only. Scores and rankings are heuristics, not posterior probabilities, proof of a key, identity or authenticity.",
                    private_input: true,
                    private_output: true,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: s.name,
                    alphabet: "Explicit exact Unicode, ASCII-letter or byte view; the request and report declare the profile.",
                    key: "Caller-owned finite keys, models and configurations. No downloads or automatic key discovery outside the requested search.",
                    key_advancement: "Reuse Core cipher conventions; ASCII letters advance Vigenere keys; punctuation is preserved.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Strict JSON <=4 MiB, text/model <=1 MiB each, 16 Mi work, 4096 results, 4 MiB serialized report. Search text <=65536 scalars. Fixed structural ceilings keep workspace below 64 MiB; a limit never implies exhaustive search.",
                rust_api: s.rust,
                python_api: s.python,
                references: &[
                    "https://web.stanford.edu/~jurafsky/slp3/3.pdf",
                    "https://pages.mtu.edu/~shene/NSF-4/Tutorial/VIG/Vig-IOC-Len.html",
                ],
            }),
        },
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: s.inputs,
            parameters: c::schema::<Empty>,
            outputs: s.outputs,
            limits: c::Limits {
                max_input_bytes: Some(4194304),
                max_output_bytes: Some(4194304),
                max_items: Some(4096),
                max_work: Some(16777216),
                max_memory_bytes: Some(67108864),
            },
            fixture: s.fixture,
        },
    }
}
static ROWS: [Row; 2] = [
    row(Spec {
        id: "analysis.language.score",
        name: "Caller-owned language-model scoring",
        command: "cryptanalysis score",
        python: "language_score",
        rust: "cube_core::cryptanalysis::scoring::score",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<scoring::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<scoring::Report>,
            required: true,
        }],
        fixture: fixtures::language_score,
    }),
    row(Spec {
        id: "analysis.period.estimate",
        name: "Explicit classical key-length evidence",
        command: "cryptanalysis period",
        python: "period_estimate",
        rust: "cube_core::cryptanalysis::period::estimate",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<period::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<period::Report>,
            required: true,
        }],
        fixture: fixtures::period_estimate,
    }),
];
const fn profiles() -> [m::Entry; 2] {
    let mut out = [ROWS[0].profile; 2];
    let mut i = 0;
    while i < 2 {
        out[i] = ROWS[i].profile;
        i += 1;
    }
    out
}
const fn contracts() -> [c::Entry; 2] {
    let mut out = [ROWS[0].contract; 2];
    let mut i = 0;
    while i < 2 {
        out[i] = ROWS[i].contract;
        i += 1;
    }
    out
}
const PROFILES: [m::Entry; 2] = profiles();
const CONTRACTS: [c::Entry; 2] = contracts();
pub static OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILES);
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILES).with_contracts(&CONTRACTS);
