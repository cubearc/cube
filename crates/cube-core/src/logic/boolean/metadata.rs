//! Discovery for the concrete tagged Boolean model; direct interfaces only.
use super::*;
use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const LIMITS: &[Parameter] = &[
    Parameter {
        name: "max_work",
        description: "All formula nodes per assignment, including both trees; no short circuit.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_WORK)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_WORK,
        },
    },
    Parameter {
        name: "max_rows",
        description: "Independent truth-row listing cap; exact full-scan counts remain complete.",
        required: false,
        default: Some(ParameterValue::Unsigned(32)),
        constraints: ParameterType::Unsigned { min: 0, max: 4096 },
    },
    Parameter {
        name: "max_memory_bytes",
        description: "Conservative scratch, retained rows and JSON bound, <=8 MiB.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_MEMORY as u64)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_MEMORY as u64,
        },
    },
    Parameter {
        name: "max_report_bytes",
        description: "Complete compact JSON upper bound, <=8 MiB; lower values may reject conservative preflight.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_REPORT as u64)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_REPORT as u64,
        },
    },
];
// Evaluation's assignment is transport in the input model contract, not a
// generic pipeline parameter. Rust functions accept explicit typed bindings.
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "logic.boolean.evaluate",
        category: "logic",
        description: "Tagged const/var/not/and/or/xor/implies/iff; explicit variable order and complete named Boolean assignment; <=12 variables, 256 nodes, depth 32, 1 MiB JSON.",
        input_type: "boolean_model_and_assignment",
        output_type: "boolean_evaluation",
        parameters: LIMITS,
        deterministic: true,
        reversible: None,
        command: "logic boolean evaluate",
        example: "cube logic boolean evaluate --text '{\"variables\":[],\"formula\":{\"op\":\"const\",\"value\":true}}' --assignment '[]'",
    },
    Descriptor {
        id: "logic.boolean.analyze",
        category: "logic",
        description: "Full bounded truth scan, exact counts/witnesses and optional implication/equivalence comparison; limited row listing is explicitly qualified.",
        input_type: "boolean_model",
        output_type: "boolean_analysis",
        parameters: LIMITS,
        deterministic: true,
        reversible: None,
        command: "logic boolean analyze",
        example: "cube logic boolean analyze --text '{\"variables\":[],\"formula\":{\"op\":\"const\",\"value\":true}}'",
    },
];
