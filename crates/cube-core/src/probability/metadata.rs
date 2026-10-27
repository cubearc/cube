use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "max_work",
        description: "Charged validation, index and execution work; exhaustion is explicit.",
        required: false,
        default: Some(ParameterValue::Unsigned(16777216)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 16777216,
        },
    },
    Parameter {
        name: "max_memory_bytes",
        description: "Conservative aggregate scratch and retained-report allocation bound.",
        required: false,
        default: Some(ParameterValue::Unsigned(33554432)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 33554432,
        },
    },
    Parameter {
        name: "max_report_bytes",
        description: "Conservative complete compact JSON report bound.",
        required: false,
        default: Some(ParameterValue::Unsigned(8388608)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 8388608,
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "probability.finite.analyze",
        category: "probability",
        description: "Exact supplied finite masses, events, conditioning and signed expectation; 4096 outcomes, 1 MiB JSON, checked u128 intermediates.",
        input_type: "finite_probability_model",
        output_type: "finite_probability_report",
        command: "probability finite analyze",
        example: "cube probability finite analyze --text '{\"outcomes\":[{\"id\":\"H\",\"weight\":1}],\"event_a\":[\"H\"]}'",
        parameters: PARAMETERS,
        deterministic: true,
        reversible: None,
    },
    Descriptor {
        id: "probability.dice.sum",
        category: "probability",
        description: "Exact bounded independent weighted dice convolution; 8 dice, 64 faces each, 4096 support points; signed i32 faces, no simulation.",
        input_type: "dice_model",
        output_type: "dice_sum_report",
        command: "probability dice sum",
        example: "cube probability dice sum --text '{\"dice\":[[{\"value\":1,\"weight\":1}]]}'",
        parameters: PARAMETERS,
        deterministic: true,
        reversible: None,
    },
    Descriptor {
        id: "probability.dice.compare",
        category: "probability",
        description: "Strict left-win, tie and loss exact masses across two independent dice sum distributions; checked u128 arithmetic and bounded pair comparisons.",
        input_type: "dice_pair",
        output_type: "dice_comparison",
        command: "probability dice compare",
        example: "cube probability dice compare --text '{\"left\":{\"dice\":[]},\"right\":{\"dice\":[]}}'",
        parameters: PARAMETERS,
        deterministic: true,
        reversible: None,
    },
];
