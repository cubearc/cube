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
const fn solver_row(s: Spec) -> Row {
    let mut result = row(s);
    result.contract.limits.max_work = Some(1048576);
    if let Some(ref mut metadata) = result.profile.metadata {
        metadata.references = &["https://aima.cs.berkeley.edu/python/csp.html"];
        metadata.resource_notes = "Observed-bijection-v1: 26..64 scalars, ciphertext <=65536 scalars/1 MiB, <=4096 exact dictionary words of <=256 scalars, <=256 cipher aliases, 1 Mi scalar/support visits, <=4096 candidates, <=64 MiB conservatively reserved workspace, <=4 MiB request/report. Unobserved keys are exact factorial classes. Report retention may stop before max_results; limits never imply contradiction.";
    }
    result
}
const fn family_row(s: Spec) -> Row {
    let mut result = row(s);
    if let Some(ref mut metadata) = result.profile.metadata {
        metadata.references = &["https://pages.mtu.edu/~shene/NSF-4/Tutorial/VIG/Vig-IOC.html"];
        metadata.resource_notes = "ASCII-classical-evidence-v1: <=1 MiB input, <=32 explicit periods in 2..64, two finite rules and at most two suggestions, <=16 Mi observation visits, <=4 MiB strict request/report. IC and column facts remain distinct from compatible-family heuristics and alternatives; no posterior probabilities or cipher identity proof.";
    }
    result
}
static ROWS: [Row; 11] = [
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
    row(Spec {
        id: "analysis.caesar.rank",
        name: "Finite Caesar candidate ranking",
        command: "cryptanalysis caesar",
        python: "caesar_rank",
        rust: "cube_core::cryptanalysis::classical::caesar",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<classical::Caesar>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<candidates::Report>,
            required: true,
        }],
        fixture: fixtures::caesar_rank,
    }),
    row(Spec {
        id: "analysis.affine.rank",
        name: "Finite Affine candidate ranking",
        command: "cryptanalysis affine",
        python: "affine_rank",
        rust: "cube_core::cryptanalysis::classical::affine",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<classical::Affine>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<candidates::Report>,
            required: true,
        }],
        fixture: fixtures::affine_rank,
    }),
    row(Spec {
        id: "analysis.vigenere.rank",
        name: "Explicit Vigenere beam search",
        command: "cryptanalysis vigenere",
        python: "vigenere_rank",
        rust: "cube_core::cryptanalysis::vigenere::rank",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<vigenere::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<vigenere::Report>,
            required: true,
        }],
        fixture: fixtures::vigenere_rank,
    }),
    row(Spec {
        id: "analysis.transposition.rank",
        name: "Explicit transposition candidates",
        command: "cryptanalysis transposition",
        python: "transposition_rank",
        rust: "cube_core::cryptanalysis::transposition::rank",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<transposition::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<candidates::Report>,
            required: true,
        }],
        fixture: fixtures::transposition_rank,
    }),
    row(Spec {
        id: "analysis.homophonic.inspect",
        name: "Exact homophonic token evidence",
        command: "cryptanalysis homophonic",
        python: "homophonic_inspect",
        rust: "cube_core::cryptanalysis::homophonic::inspect",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<homophonic::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<homophonic::Report>,
            required: true,
        }],
        fixture: fixtures::homophonic_inspect,
    }),
    row(Spec {
        id: "analysis.crib.verify",
        name: "Explicit known-plaintext verification",
        command: "cryptanalysis crib",
        python: "crib_verify",
        rust: "cube_core::cryptanalysis::crib::verify",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<crib::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<crib::Report>,
            required: true,
        }],
        fixture: fixtures::crib_verify,
    }),
    row(Spec {
        id: "analysis.frequency.compare",
        name: "Explicit Pearson frequency distance",
        command: "cryptanalysis frequency",
        python: "frequency_compare",
        rust: "cube_core::cryptanalysis::frequency::compare",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<frequency::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<frequency::Report>,
            required: true,
        }],
        fixture: fixtures::frequency_compare,
    }),
    solver_row(Spec {
        id: "analysis.substitution.solve",
        name: "Exact bounded substitution constraints",
        command: "cryptanalysis substitution",
        python: "substitution_solve",
        rust: "cube_core::cryptanalysis::substitution::solve",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<substitution::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<substitution::Report>,
            required: true,
        }],
        fixture: fixtures::substitution_solve,
    }),
    family_row(Spec {
        id: "analysis.family.suggest",
        name: "Evidence-linked classical family suggestions",
        command: "cryptanalysis families",
        python: "family_suggest",
        rust: "cube_core::cryptanalysis::family::suggest",
        inputs: &[c::Port {
            name: "request",
            semantic_type: "json",
            schema: c::schema::<family::Request>,
            required: true,
        }],
        outputs: &[c::Port {
            name: "report",
            semantic_type: "analysis.report",
            schema: c::schema::<family::Report>,
            required: true,
        }],
        fixture: fixtures::family_suggest,
    }),
];
const fn profiles() -> [m::Entry; 11] {
    let mut out = [ROWS[0].profile; 11];
    let mut i = 0;
    while i < 11 {
        out[i] = ROWS[i].profile;
        i += 1;
    }
    out
}
const fn contracts() -> [c::Entry; 11] {
    let mut out = [ROWS[0].contract; 11];
    let mut i = 0;
    while i < 11 {
        out[i] = ROWS[i].contract;
        i += 1;
    }
    out
}
const PROFILES: [m::Entry; 11] = profiles();
const CONTRACTS: [c::Entry; 11] = contracts();
pub static OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILES);
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILES).with_contracts(&CONTRACTS);
