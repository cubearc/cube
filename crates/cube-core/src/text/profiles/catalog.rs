//! Source-visible named-port operations and their reviewed known answers.
use super::*;
use crate::operations::{self as o, contracts as c};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NoParameters {}

struct Row {
    contract: c::Entry,
    execute: fn(Value, &'static str) -> Result<Value, OperationError>,
}

const fn declare(id: &'static str, mut row: Row) -> Row {
    row.contract.descriptor.id = id;
    row
}

macro_rules! profile {
    ($name:literal, $slug:literal, $description:literal,
     $request:ty => $report:ty, $run:path,
     ($input:expr, $output:expr, $items:expr, $work:expr, $memory:expr),
     $sample:literal => $answer:literal) => {
        Row {
            contract: c::Entry {
                operation_version: 1,
                descriptor: o::Descriptor {
                    id: "",
                    category: "text",
                    description: $description,
                    input_type: "json",
                    output_type: "json",
                    parameters: &[],
                    deterministic: true,
                    reversible: Some(false),
                    command: concat!("text profile ", $slug),
                    example: concat!("cube text profile ", $slug, " --text '", $sample, "'"),
                },
                inputs: &[c::Port {
                    name: "request",
                    semantic_type: concat!("text.", $name, ".request"),
                    schema: c::schema::<$request>,
                    required: true,
                }],
                parameters: c::schema::<NoParameters>,
                outputs: &[c::Port {
                    name: "report",
                    semantic_type: concat!("text.", $name, ".report"),
                    schema: c::schema::<$report>,
                    required: true,
                }],
                limits: c::Limits {
                    max_input_bytes: Some($input),
                    max_output_bytes: Some($output),
                    max_items: Some($items),
                    max_work: Some($work),
                    max_memory_bytes: Some($memory),
                },
                fixture: || Ok(c::Fixture {
                    inputs: json!({"request": serde_json::from_str::<Value>($sample).expect("fixture request")}),
                    parameters: json!({}),
                    outputs: json!({"report": serde_json::from_str::<Value>($answer).expect("fixture answer")}),
                }),
            },
            execute: |value, operation| {
                let request: $request = serde_json::from_value(value)
                    .map_err(|_| invalid("request does not match the selected text profile")
                        .operation(operation))?;
                let report = $run(&request).map_err(|e| e.operation(operation))?;
                serde_json::to_value(report)
                    .map_err(|_| invalid("text report could not be serialized").operation(operation))
            },
        }
    };
}

const ROWS: [Row; 11] = [
    declare(
        "text.scalar_weave",
        profile!("scalar_weave", "scalar-weave", "Pad exact scalars or interleave and recover explicit lanes.",
    scalar_weave::Request => scalar_weave::Report, scalar_weave::apply,
    (7 << 20, 2 << 20, 65536, 1 << 20, 8 << 20),
    r#"{"action":"interleave","mode":"ragged","lanes":["Aé","B","","🧊C"]}"#
    => r#"{"action":"interleave","text":"AB🧊éC","lengths":[2,1,0,2]}"#),
    ),
    declare(
        "text.select_words",
        profile!("select_words", "select-words", "Select signed whole-word indexes with exact source spans.",
    lexical::SelectRequest => lexical::SelectReport<'static>, lexical::select_words,
    (9 << 20, 8 << 20, 65536, 8 << 20, 16 << 20),
    r#"{"text":"é! B","indices":[-1,1,1,0],"out_of_range":"skip"}"#
    => r#"{"selected":[{"index":-1,"word_index":1,"text":"B","byte_start":6,"byte_end":7,"scalar_start":3,"scalar_end":4},{"index":1,"word_index":0,"text":"é!","byte_start":0,"byte_end":3,"scalar_start":0,"scalar_end":2},{"index":1,"word_index":0,"text":"é!","byte_start":0,"byte_end":3,"scalar_start":0,"scalar_end":2}],"omitted":[{"position":3,"index":0}]}"#),
    ),
    declare(
        "text.sort",
        profile!("sort", "sort", "Sort exact scalars or tokens and preserve stable source indexes.",
    lexical::SortRequest => lexical::SortReport<'static>, lexical::sort,
    (7 << 20, 8 << 20, 65536, 8 << 20, 16 << 20),
    r#"{"unit":"tokens","tokens":["b","a","A","A","ä"]}"#
    => r#"{"unit":"tokens","tokens":["A","A","a","b","ä"],"original_indices":[2,3,1,0,4]}"#),
    ),
    declare(
        "text.unicode_transform",
        profile!("unicode_transform", "unicode-transform", "Apply explicit ASCII casing, Unicode case folding or category filtering.",
    case::Request => case::Report, case::apply,
    (7 << 20, 8 << 20, 1 << 20, 8 << 20, 16 << 20),
    r#"{"transform":"casefold","text":"Fuß İΣ"}"#
    => r#"{"text":"fuss i̇σ","profile":"casefold_full","unicode_version":[17,0,0]}"#),
    ),
    declare(
        "text.graphemes",
        profile!("graphemes", "graphemes", "Segment or reverse default extended Unicode grapheme clusters.",
    graphemes::Request => graphemes::Report, graphemes::apply,
    (7 << 20, 8 << 20, 65536, 4 << 20, 16 << 20),
    r#"{"action":"reverse","text":"á🧑‍💻b"}"#
    => r#"{"action":"reverse","text":"b🧑‍💻á","unicode_version":[17,0,0],"uax29_revision":47}"#),
    ),
    declare(
        "text.regex",
        profile!("regex", "regex", "Match, capture or split UTF-8 with a bounded Rust regex engine.",
    regex::Request => regex::Report<'static>, regex::search,
    (7 << 20, 8 << 20, 65536, 64 << 20, 32 << 20),
    r#"{"action":"exists","text":"é","pattern":"é"}"#
    => r#"{"action":"exists","exists":true,"profile":"regex-syntax-0.8.11/pikevm-0.4.18","work_bound":168}"#),
    ),
    declare(
        "text.tokenize",
        profile!("tokenize", "tokenize", "Return existing Unicode-whitespace tokens with exact source spans.",
    regex::TokenizeRequest => regex::TokenizeReport<'static>, regex::tokenize,
    (7 << 20, 8 << 20, 65536, 8 << 20, 16 << 20),
    r#"{"profile":"unicode_whitespace","text":"ä! B"}"#
    => r#"{"profile":"unicode_whitespace","unicode_version":[17,0,0],"tokens":[{"text":"ä!","byte_start":0,"byte_end":3,"scalar_start":0,"scalar_end":2},{"text":"B","byte_start":6,"byte_end":7,"scalar_start":3,"scalar_end":4}]}"#),
    ),
    declare(
        "text.phonetic",
        profile!("phonetic", "phonetic", "Encode explicit one-name or batch phonetic keys.",
    phonetic::Request => phonetic::Report, phonetic::encode,
    (512 << 10, 1 << 20, 256, 16 << 20, 1 << 20),
    r#"{"method":"soundex_nara","names":["Washington","Lee","Ashcraft"]}"#
    => r#"{"method":"soundex_nara","profile":"nara-census-four-character-v1","strict":null,"max_code_length":null,"keys":[{"key":"W252"},{"key":"L000"},{"key":"A261"}]}"#),
    ),
    declare(
        "text.match_rating",
        profile!("match_rating", "match-rating", "Derive Match Rating Approach keys or compare exact ASCII names.",
    match_rating::Request => match_rating::Report, match_rating::apply,
    (4096, 4096, 2, 1 << 20, 1 << 20),
    r#"{"action":"compare","left":"Miller","right":"Miller"}"#
    => r#"{"action":"compare","profile":"apache-commons-codec-1.15/cube-ascii-v1","left_key":"MLR","right_key":"MLR","unmatched_count":0,"rating":6,"minimum":4,"matches":true}"#),
    ),
    declare(
        "text.daitch_mokotoff",
        profile!("daitch_mokotoff", "daitch-mokotoff", "Return all bounded Daitch–Mokotoff phonetic alternatives.",
    daitch_mokotoff::Request => daitch_mokotoff::Report, daitch_mokotoff::encode,
    (8192, 1 << 20, 4096, 65536, 8 << 20),
    r#"{"name":"Auerbach"}"#
    => r#"{"profile":"apache-commons-codec-1.15-dm","folding":false,"codes":["097400","097500"]}"#),
    ),
    declare(
        "text.beider_morse",
        profile!("beider_morse", "beider-morse", "Compute bounded Beider–Morse alternatives with explicit language selection.",
    beider_morse::Request => beider_morse::Report, beider_morse::encode,
    (8192, 1 << 20, 4096, 1 << 20, 32 << 20),
    r#"{"name":"Santos","name_type":"sephardic","rule_type":"exact","language":{"mode":"explicit","languages":["portuguese"]}}"#
    => r#"{"profile":"apache-commons-codec-1.15-bm/cube-v1","profile_sha256":"834d0ae444e2aea76b808c242d7541ff1585c1fc29ae2ba47cd758974ca9a4a3","name_type":"sephardic","rule_type":"exact","concatenate":true,"encoding":"santoS|santuS","groups":[{"path":[],"table_language":"portuguese","languages":["portuguese"],"alternatives":["santoS","santuS"]}],"work_units":351,"peak_alternatives":2}"#),
    ),
];

const fn contracts() -> [c::Entry; ROWS.len()] {
    let mut entries = [ROWS[0].contract; ROWS.len()];
    let mut index = 1;
    while index < ROWS.len() {
        entries[index] = ROWS[index].contract;
        index += 1;
    }
    entries
}

const fn descriptors() -> [o::Descriptor; ROWS.len()] {
    let mut entries = [ROWS[0].contract.descriptor; ROWS.len()];
    let mut index = 1;
    while index < ROWS.len() {
        entries[index] = ROWS[index].contract.descriptor;
        index += 1;
    }
    entries
}

pub const OPERATIONS: &[o::Descriptor] = &descriptors();
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &[]).with_contracts(&contracts());

pub(super) fn invoke(operation: &str, value: Value) -> Result<Value, OperationError> {
    let row = ROWS
        .iter()
        .find(|row| row.contract.descriptor.id == operation)
        .ok_or_else(|| invalid("unknown text profile"))?;
    (row.execute)(value, row.contract.descriptor.id)
}
