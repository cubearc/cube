//! Module-owned operation discovery; no pipeline execution.
use super::*;
use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const INPUT: Parameter = Parameter {
    name: "input",
    description: "Explicit program input bytes, <=1 MiB; never implicit stdin in Core.",
    required: false,
    default: Some(ParameterValue::Bytes(&[])),
    constraints: ParameterType::Bytes { min_length: 0 },
};
const WIDTH: Parameter = Parameter {
    name: "cell_bits",
    description: "Unsigned wrapping width; input zero-extends bytes, output emits low eight bits.",
    required: false,
    default: Some(ParameterValue::Unsigned(8)),
    constraints: ParameterType::UnsignedChoices {
        choices: &[8, 16, 32],
    },
};
const EOF: Parameter = Parameter {
    name: "eof",
    description: "Explicit policy when program input is exhausted.",
    required: false,
    default: Some(ParameterValue::Text("error")),
    constraints: ParameterType::Text {
        choices: &["error", "zero", "unchanged"],
    },
};
const STEPS: Parameter = Parameter {
    name: "max_instructions",
    description: "Maximum dispatched commands, including loop tests and failed effects.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_INSTRUCTIONS)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_INSTRUCTIONS,
    },
};
const OUTPUT: Parameter = Parameter {
    name: "max_output_bytes",
    description: "Maximum output bytes; zero permits no output.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_OUTPUT_BYTES as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_OUTPUT_BYTES as u64,
    },
};
const MEMORY: Parameter = Parameter {
    name: "max_memory_bytes",
    description: "Conservative total allocation and compact JSON bound, <=8 MiB.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_MEMORY_BYTES as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_MEMORY_BYTES as u64,
    },
};
const TAPE_START: Parameter = Parameter {
    name: "tape_start",
    description: "Exact final tape range start, indexed from zero.",
    required: false,
    default: Some(ParameterValue::Unsigned(0)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: TAPE_CELLS as u64,
    },
};
const TAPE_LEN: Parameter = Parameter {
    name: "tape_len",
    description: "Exact number of cells; start+length must not exceed 30000.",
    required: false,
    default: Some(ParameterValue::Unsigned(0)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: TAPE_CELLS as u64,
    },
};
const TRACE_RECORDS: Parameter = Parameter {
    name: "max_trace_records",
    description: "Zero disables traces; exhaustion marks trace_complete=false and continues execution.",
    required: false,
    default: Some(ParameterValue::Unsigned(0)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_TRACE_RECORDS as u64,
    },
};
const TRACE_BYTES: Parameter = Parameter {
    name: "max_trace_bytes",
    description: "Explicit in-memory trace byte budget, paired with positive trace records.",
    required: false,
    default: Some(ParameterValue::Unsigned(0)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_TRACE_BYTES as u64,
    },
};
const REPORT_BYTES: Parameter = Parameter {
    name: "max_report_bytes",
    description: "Conservative compact JSON report size cap; preflight never silently truncates evidence.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_REPORT_BYTES as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_REPORT_BYTES as u64,
    },
};
const WIDTHS: Parameter = Parameter {
    name: "widths",
    description: "Nonempty unique subset of 8/16/32 (other integers in this range fail); ascending execution order.",
    required: true,
    default: None,
    constraints: ParameterType::UnsignedList {
        min: 8,
        max: 32,
        min_length: 1,
        max_length: 3,
        unique: true,
    },
};
const TOTAL_STEPS: Parameter = Parameter {
    name: "total_instructions",
    description: "Shared maximum dispatches across all widths.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_INSTRUCTIONS)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_INSTRUCTIONS,
    },
};
const TOTAL_OUTPUT: Parameter = Parameter {
    name: "total_output_bytes",
    description: "Shared maximum retained output bytes across all widths.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_OUTPUT_BYTES as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_OUTPUT_BYTES as u64,
    },
};
const TOTAL_MEMORY: Parameter = Parameter {
    name: "total_memory_bytes",
    description: "Aggregate memory bound including retained runs and compact JSON, <=8 MiB.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_MEMORY_BYTES as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_MEMORY_BYTES as u64,
    },
};
/// Direct interfaces only: no pipeline adapter or host capabilities.
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: OP,
        category: "esolang",
        description: "Strict bounded Brainfuck, <=1 MiB source/input/output, 65536 commands, depth 256, fixed 30000 cells; exact bytes only on completion.",
        input_type: "text",
        output_type: "bytes",
        parameters: &[INPUT, WIDTH, EOF, STEPS, OUTPUT, MEMORY],
        deterministic: true,
        reversible: None,
        command: "esolang brainfuck run",
        example: "cube esolang brainfuck run --code '++++++++[>++++++++<-]>+.' --output text",
    },
    Descriptor {
        id: "esolang.brainfuck.report",
        category: "esolang",
        description: "Bounded execution summary, exact tape range and optional post-dispatch traces; incomplete output is labeled.",
        input_type: "text",
        output_type: "brainfuck_report",
        parameters: &[
            INPUT,
            WIDTH,
            EOF,
            STEPS,
            OUTPUT,
            MEMORY,
            TAPE_START,
            TAPE_LEN,
            TRACE_RECORDS,
            TRACE_BYTES,
            REPORT_BYTES,
        ],
        deterministic: true,
        reversible: None,
        command: "esolang brainfuck report",
        example: "cube esolang brainfuck report --code '-.' --cell-bits 16 --tape-len 1",
    },
    Descriptor {
        id: "esolang.brainfuck.compare",
        category: "esolang",
        description: "Compare isolated 8/16/32-bit runs with shared aggregate work/output/memory budgets; incomplete comparisons are explicit.",
        input_type: "text",
        output_type: "brainfuck_comparison",
        parameters: &[
            INPUT,
            WIDTHS,
            EOF,
            STEPS,
            OUTPUT,
            MEMORY,
            TOTAL_STEPS,
            TOTAL_OUTPUT,
            TOTAL_MEMORY,
        ],
        deterministic: true,
        reversible: None,
        command: "esolang brainfuck compare",
        example: "cube esolang brainfuck compare --code '-.' --widths 8,16,32",
    },
];
