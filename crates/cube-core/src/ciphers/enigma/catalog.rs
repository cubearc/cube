use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};

#[derive(Serialize, JsonSchema)]
#[schemars(transparent)]
struct Input(#[schemars(length(max = 1048576), extend("x-cube-max-utf8-bytes" = 1048576))] String);

const DESCRIPTION: &str = "Reciprocal historical Enigma I/M3 with explicit settings, final windows and optional bounded state trace.";
const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Exact historical machine profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["enigma_i", "m3"],
        },
    },
    o::Parameter {
        name: "rotors",
        description: "Distinct left-to-right rotors; Enigma I accepts I-V only.",
        required: true,
        default: None,
        constraints: o::ParameterType::TextList {
            choices: &["I", "II", "III", "IV", "V", "VI", "VII", "VIII"],
            min_length: 3,
            max_length: 3,
            unique: true,
        },
    },
    o::Parameter {
        name: "reflector",
        description: "Wide historical reflector; no thin/M4 profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["B", "C"],
        },
    },
    o::Parameter {
        name: "rings",
        description: "Left-to-right ring settings; 1 corresponds to A.",
        required: true,
        default: None,
        constraints: o::ParameterType::UnsignedList {
            min: 1,
            max: 26,
            min_length: 3,
            max_length: 3,
            unique: false,
        },
    },
    o::Parameter {
        name: "windows",
        description: "Exactly three uppercase ASCII A-Z initial windows, left to right.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "plugboard",
        description: "Zero to ten disjoint uppercase ASCII letter pairs.",
        required: true,
        default: None,
        constraints: o::ParameterType::TextList {
            choices: &[],
            min_length: 0,
            max_length: 10,
            unique: true,
        },
    },
    o::Parameter {
        name: "text_mode",
        description: "Strict A-Z or explicit case/other-scalar preservation; step on ASCII letters only.",
        required: false,
        default: Some(o::ParameterValue::Text("strict")),
        constraints: o::ParameterType::Text {
            choices: &["strict", "preserve"],
        },
    },
    o::Parameter {
        name: "trace",
        description: "Return at most 65536 post-step letter states; fail instead of truncating.",
        required: false,
        default: Some(o::ParameterValue::Boolean(false)),
        constraints: o::ParameterType::Boolean,
    },
];
const PROFILED: [m::Entry; 1] = [m::Entry {
    descriptor: o::Descriptor {
        id: ID,
        category: "ciphers",
        description: DESCRIPTION,
        input_type: "text",
        output_type: "json",
        parameters: PARAMETERS,
        deterministic: true,
        reversible: Some(true),
        command: "cipher enigma",
        example: "cube cipher enigma --parameters settings.json --text AAAAA",
    },
    metadata: Some(m::Metadata {
        schema_version: 1,
        canonical_name: "Enigma I / M3",
        family: "historical_rotor",
        aliases: &[],
        security: m::Security {
            classification: m::Classification::HistoricalCipher,
            limitations: "Historical education and interoperability only; no modern confidentiality or authentication. Settings and state traces can disclose key material.",
            private_input: true,
            private_output: true,
            sensitive_parameters: &["rotors", "rings", "windows", "plugboard"],
        },
        profile: m::Profile {
            variant: DESCRIPTION,
            alphabet: "Strict ASCII A-Z by default; explicit preserve wrapper retains ASCII case and other exact Unicode scalars.",
            key: "Required profile, left-to-right rotors/rings/windows, reflector and 0..10 disjoint plug pairs; rings 1..26.",
            key_advancement: "Pre-keypress rightmost-fast pawl stepping, middle double-step, index-ring notches unaffected by Ringstellung; letters only.",
            block_bits: None,
            output_bits: None,
        },
        resources: &[],
        resource_notes: "Input/output text <=1 MiB; settings JSON <=4096 bytes; optional trace <=65536 post-step states and <4 MiB serialized JSON. Six fixed rotor lookups per letter. No pipeline adapter; typed direct Play integration.",
        rust_api: "cube_core::ciphers::enigma::transform",
        python_api: "enigma",
        references: &[
            "https://www.cryptomuseum.com/crypto/enigma/wiring.htm",
            "https://www.cryptomuseum.com/people/hamer/files/double_stepping.pdf",
            "https://www.enigma.hoerenberg.com/index.php?cat=Historical%20Messages",
        ],
    }),
}];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const CONTRACTS: &[c::Entry] = &[c::Entry {
    operation_version: 1,
    descriptor: OPERATIONS[0],
    inputs: &[c::Port {
        name: "text",
        semantic_type: "text",
        schema: c::schema::<Input>,
        required: true,
    }],
    parameters: c::schema::<Parameters>,
    outputs: &[
        c::Port {
            name: "text",
            semantic_type: "text",
            schema: || c::field::<Report>("text"),
            required: true,
        },
        c::Port {
            name: "final_windows",
            semantic_type: "text",
            schema: || c::field::<Report>("final_windows"),
            required: true,
        },
        c::Port {
            name: "trace",
            semantic_type: "json",
            schema: || c::field::<Report>("trace"),
            required: true,
        },
    ],
    limits: c::Limits {
        max_input_bytes: Some(MAX_INPUT as u64),
        max_output_bytes: Some(10 * 1024 * 1024),
        max_items: Some(MAX_TRACE as u64),
        max_work: None,
        max_memory_bytes: None,
    },
    fixture,
}];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);

fn fixture() -> Result<c::Fixture, OperationError> {
    let parameters = serde_json::json!({"profile":"enigma_i","rotors":["I","II","III"],"reflector":"B","rings":[1,1,1],"windows":"AAA","plugboard":[]});
    let p: Parameters =
        serde_json::from_value(parameters.clone()).expect("typed synthetic settings");
    let report = transform("AAAAA", &p)?;
    assert_eq!(report.text, "BDZGO");
    assert_eq!(report.final_windows, "AAF");
    assert!(report.trace.is_empty());
    Ok(c::Fixture {
        inputs: serde_json::json!({"text":"AAAAA"}),
        parameters,
        outputs: serde_json::to_value(report).expect("typed report"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn actual_result_and_invalid_boundaries_match_contract() {
        let c = c::describe(&OPERATIONS[0]).unwrap();
        let f = fixture().unwrap();
        c.validate_fixture(&f).unwrap();
        for (key, value) in [
            ("rings", json!([0, 1, 1])),
            ("rings", json!([1, 27, 1])),
            ("windows", json!("aaA")),
            ("plugboard", json!(["A"])),
            ("profile", json!("m4")),
            ("extra", json!(true)),
        ] {
            let mut p = f.parameters.clone();
            p[key] = value;
            assert!(c.validate_parameters(&p).is_err(), "{key}");
        }
        assert!(
            c.validate_inputs(&json!({"text":"A".repeat(MAX_INPUT+1)}))
                .is_err()
        );
        assert!(!c.policy.automatic_capture);
        assert!(c.policy.private_output);
        assert!(c.pipeline_adapter.is_none());
    }
}
