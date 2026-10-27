//! Module-owned graph discovery. Structured results have no pipeline adapters.
use super::*;
use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const WORK: Parameter = Parameter {
    name: "max_work",
    description: "Charged graph construction and vertex/edge visits; exhaustion is a resource error.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_WORK)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_WORK,
    },
};
const MEMORY: Parameter = Parameter {
    name: "max_memory_bytes",
    description: "Conservative graph/index/traversal/report allocation ceiling, <=32 MiB.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_MEMORY as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_MEMORY as u64,
    },
};
const REPORT: Parameter = Parameter {
    name: "max_report_bytes",
    description: "Complete compact report ceiling, <=8 MiB.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_REPORT as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_REPORT as u64,
    },
};
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "graph.components",
        category: "graph",
        description: "Weak components of explicit ordered directed/undirected multigraph; includes isolates, <=4096 vertices/32768 edges/2 MiB input.",
        input_type: "graph_definition",
        output_type: "graph_components",
        parameters: &[WORK, MEMORY, REPORT],
        deterministic: true,
        reversible: None,
        command: "graph components",
        example: "cube graph components --text '{\"vertices\":[\"a\"],\"directed\":false,\"edges\":[]}'",
    },
    Descriptor {
        id: "graph.shortest_path",
        category: "graph",
        description: "Unweighted BFS path with vertex/edge ID evidence, edge input order ties; null means no route.",
        input_type: "graph_definition",
        output_type: "graph_path",
        parameters: &[
            Parameter {
                name: "source",
                description: "Explicit existing source vertex.",
                required: true,
                default: None,
                constraints: ParameterType::Text { choices: &[] },
            },
            Parameter {
                name: "target",
                description: "Explicit existing target vertex.",
                required: true,
                default: None,
                constraints: ParameterType::Text { choices: &[] },
            },
            WORK,
            MEMORY,
            REPORT,
        ],
        deterministic: true,
        reversible: None,
        command: "graph shortest-path",
        example: "cube graph shortest-path --source a --target a --text '{\"vertices\":[\"a\"],\"directed\":false,\"edges\":[]}'",
    },
    Descriptor {
        id: "graph.cycle",
        category: "graph",
        description: "Cycle presence with exact ordered edge/vertex witness; loops and parallel undirected edges count.",
        input_type: "graph_definition",
        output_type: "graph_cycle",
        parameters: &[WORK, MEMORY, REPORT],
        deterministic: true,
        reversible: None,
        command: "graph cycle",
        example: "cube graph cycle --text '{\"vertices\":[],\"directed\":true,\"edges\":[]}'",
    },
    Descriptor {
        id: "graph.euler_trail",
        category: "graph",
        description: "Hierholzer edge-once trail/circuit with exact orientation, degree/connectivity rules and requested-start validation.",
        input_type: "graph_definition",
        output_type: "graph_euler_trail",
        parameters: &[
            Parameter {
                name: "start",
                description: "Optional existing start; must satisfy directed/undirected degree conditions.",
                required: false,
                default: None,
                constraints: ParameterType::Text { choices: &[] },
            },
            WORK,
            MEMORY,
            REPORT,
        ],
        deterministic: true,
        reversible: None,
        command: "graph euler-trail",
        example: "cube graph euler-trail --text '{\"vertices\":[],\"directed\":false,\"edges\":[]}'",
    },
];
