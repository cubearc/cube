//! Named-port contracts and reviewed known answers for sequence analysis.
use super::*;
use crate::operations::{self as o, contracts as c};
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NoParameters {}

macro_rules! sequence_operations {
    ($($name:literal, $slug:literal, $description:literal,
       $request:ty => $report:ty, $run:path,
       ($input:expr, $output:expr, $items:expr, $work:expr, $memory:expr),
       $sample:literal => $answer:literal;)*) => {
        pub const OPERATIONS: &[o::Descriptor] = &[$(o::Descriptor {
            id: concat!("sequence.", $name),
            category: "sequence",
            description: $description,
            input_type: "json",
            output_type: "json",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: concat!("sequence ", $slug),
            example: concat!("cube sequence ", $slug, " --text '", $sample, "'"),
        }),*];

        pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &[])
            .with_contracts(&[$(c::Entry {
                operation_version: 1,
                descriptor: descriptor(concat!("sequence.", $name)),
                inputs: &[c::Port {
                    name: "request",
                    semantic_type: concat!("sequence.", $name, ".request"),
                    schema: c::schema::<$request>,
                    required: true,
                }],
                parameters: c::schema::<NoParameters>,
                outputs: &[c::Port {
                    name: "report",
                    semantic_type: concat!("sequence.", $name, ".report"),
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
            }),*]);

        pub(super) fn invoke(operation: &str, value: Value) -> Result<Value, OperationError> {
            match operation {
                $(concat!("sequence.", $name) => {
                    let request: $request = serde_json::from_value(value)
                        .map_err(|_| invalid("request does not match the selected sequence profile")
                            .operation(concat!("sequence.", $name)))?;
                    let report: $report = $run(&request)
                        .map_err(|e| e.operation(concat!("sequence.", $name)))?;
                    serde_json::to_value(report)
                        .map_err(|_| invalid("sequence report could not be serialized"))
                }),*
                _ => Err(invalid("unknown sequence operation")),
            }
        }
    };
}

const fn descriptor(id: &str) -> o::Descriptor {
    let mut i = 0;
    while i < OPERATIONS.len() {
        if same(OPERATIONS[i].id.as_bytes(), id.as_bytes()) {
            return OPERATIONS[i];
        }
        i += 1;
    }
    panic!("unknown sequence descriptor")
}

const fn same(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut i = 0;
    while i < left.len() {
        if left[i] != right[i] {
            return false;
        }
        i += 1;
    }
    true
}

sequence_operations! {
    "hamming", "hamming", "Count differing byte, scalar or packed-bit positions.",
    HammingRequest => HammingReport, hamming,
    (46 << 20, 1024, 16 << 20, 1 << 25, 8192),
    r#"{"unit":"bits","left":"/wA=","right":"AIA="}"#
    => r#"{"differing_units":9,"total_units":16,"normalized":0.5625}"#;

    "edit_distance", "edit-distance", "Compare exact sequences with an explicit edit or transposition profile.",
    EditRequest => EditReport, edit_distance,
    (13 << 20, 1024, 65536, 1 << 24, 64 << 20),
    r#"{"operands":{"unit":"scalars","left":"CA","right":"ABC"},"method":"dl"}"#
    => r#"{"method":"dl","distance":2,"above_threshold":false}"#;

    "jaro", "jaro", "Score exact scalar sequences with Jaro or Jaro–Winkler.",
    JaroRequest => JaroReport, jaro,
    (13631488, 1024, 65536, 16777216, 8388608),
    r#"{"left":"a","right":"a"}"#
    => r#"{"method":"jaro","similarity":1.0,"distance":0.0,"scale":0.1,"threshold":0.7}"#;

    "lcs", "lcs", "Recover a longest common subsequence with paired source positions.",
    Pair => LcsReport, lcs,
    (13631488, 4194304, 65536, 16777216, 16777216),
    r#"{"unit":"scalars","left":"xé","right":"é"}"#
    => r#"{"length":1,"subsequence":{"unit":"scalars","text":"é"},"positions":[{"length":1,"left":{"start":1,"end":2,"byte_start":1,"byte_end":3},"right":{"start":0,"end":1,"byte_start":0,"byte_end":2}}]}"#;

    "common_spans", "common-spans", "Locate exact longest common prefix, suffix and contiguous substring.",
    Pair => CommonSpans, common_spans,
    (13631488, 4194304, 65536, 8388608, 8388608),
    r#"{"unit":"scalars","left":"ab","right":"a"}"#
    => r#"{"prefix":{"length":1,"left":{"start":0,"end":1,"byte_start":0,"byte_end":1},"right":{"start":0,"end":1,"byte_start":0,"byte_end":1}},"suffix":{"length":0,"left":{"start":0,"end":0,"byte_start":0,"byte_end":0},"right":{"start":0,"end":0,"byte_start":0,"byte_end":0}},"substring":{"length":1,"left":{"start":0,"end":1,"byte_start":0,"byte_end":1},"right":{"start":0,"end":1,"byte_start":0,"byte_end":1}}}"#;

    "ratcliff_obershelp", "ratcliff-obershelp", "Score common contiguous blocks with deterministic tie-breaking.",
    Pair => RatcliffReport, ratcliff_obershelp,
    (13631488, 4194304, 16384, 8388608, 16777216),
    r#"{"unit":"scalars","left":"Xa","right":"aY"}"#
    => r#"{"similarity":0.5,"distance":0.5,"blocks":[{"length":1,"left":{"start":1,"end":2,"byte_start":1,"byte_end":2},"right":{"start":0,"end":1,"byte_start":0,"byte_end":1}}]}"#;

    "monge_elkan", "monge-elkan", "Match ordered token arrays by directional best similarity.",
    MongeRequest => MongeReport, monge_elkan,
    (409600, 1048576, 256, 8388608, 16777216),
    r#"{"left":["a","a"],"right":["b","a"],"base":"jaro"}"#
    => r#"{"base":"jaro","direction":"left_to_right","similarity":1.0,"matches":[{"index":1,"similarity":1.0},{"index":1,"similarity":1.0}],"reverse_matches":null}"#;

    "align", "align", "Align two exact sequences with global or local integer gap scoring.",
    AlignRequest => AlignReport, align,
    (13631488, 4194304, 16384, 16777216, 67108864),
    r#"{"operands":{"unit":"scalars","left":"A","right":"A"},"mode":"global"}"#
    => r#"{"score":2,"left":{"start":0,"end":1,"byte_start":0,"byte_end":1},"right":{"start":0,"end":1,"byte_start":0,"byte_end":1},"runs":[{"kind":"equal","left":{"start":0,"end":1,"byte_start":0,"byte_end":1},"right":{"start":0,"end":1,"byte_start":0,"byte_end":1}}]}"#;

    "dtw", "dtw", "Compute a closed-endpoint dynamic time warping cost and path.",
    DtwRequest => DtwReport, dtw,
    (4194304, 4194304, 65536, 8388608, 33554432),
    r#"{"left":[0,2],"right":[0,1]}"#
    => r#"{"cost":1.0,"path":[[0,0],[1,1]]}"#;

    "set_similarity", "set-similarity", "Compare exact token sets or overlapping scalar q-gram counts.",
    inventory::SetRequest => inventory::SetReport, inventory::set_similarity,
    (13631488, 4194304, 65536, 16777216, 33554432),
    r#"{"input":{"kind":"tokens","left":["a","a","b"],"right":["a"]},"method":"jaccard"}"#
    => r#"{"method":"jaccard","intersection":1,"union":2,"left_only":1,"right_only":0,"fraction":{"numerator":"1","denominator":"2"},"similarity":0.5,"jaccard_distance":0.5,"l1":null}"#;

    "vector_similarity", "vector-similarity", "Compute cosine, Kullback–Leibler or Jensen–Shannon scores.",
    vector::VectorRequest => vector::VectorReport, vector::vector_similarity,
    (2097152, 1024, 10000, 1048576, 8388608),
    r#"{"input":{"kind":"dense","left":[1,0],"right":[0,1]},"method":"kl"}"#
    => r#"{"method":"kl","status":"positive_infinity","value":null,"reason":"positive_mass_over_zero"}"#;

    "tfidf_model", "tfidf-model", "Build an exact document-frequency model from supplied tokens.",
    tfidf::Corpus => tfidf::TfidfModel, tfidf::tfidf_model,
    (8388608, 4194304, 65536, 1048576, 33554432),
    r#"{"documents":[["a","a"],["b"]]}"#
    => r#"{"profile":"raw_tf_smooth_idf_l2_v1","documents":2,"frequencies":[{"token":"a","df":1},{"token":"b","df":1}],"sha256":"a18846f1267e910f42e78e19aad51896fb028c1e7b0637e7efe7e2f186e3160b"}"#;

    "tfidf_compare", "tfidf-compare", "Compare two documents under a verified TF-IDF model.",
    tfidf::TfidfRequest => tfidf::TfidfReport, tfidf::tfidf_compare,
    (13631488, 4194304, 65536, 1048576, 33554432),
    r#"{"model":{"profile":"raw_tf_smooth_idf_l2_v1","documents":2,"frequencies":[{"token":"a","df":1},{"token":"b","df":1}],"sha256":"a18846f1267e910f42e78e19aad51896fb028c1e7b0637e7efe7e2f186e3160b"},"left":["a"],"right":["a","a"]}"#
    => r#"{"profile":"raw_tf_smooth_idf_l2_v1","model_sha256":"a18846f1267e910f42e78e19aad51896fb028c1e7b0637e7efe7e2f186e3160b","similarity":1.0,"reason":null}"#;

    "exact_search", "exact-search", "Find overlapping exact byte positions with a named classic algorithm.",
    search::ExactRequest => search::ExactReport, search::exact_search,
    (25165824, 8388608, 65536, 67108864, 16777216),
    r#"{"haystack":"YmFuYW5h","pattern":"YW5h","algorithm":"kmp"}"#
    => r#"{"algorithm":"kmp","count":2,"complete":true,"matches":[{"start":1,"end":4},{"start":3,"end":6}]}"#;

    "multi_search", "multi-search", "Search ordered byte patterns with a bounded Aho–Corasick automaton.",
    search::MultiRequest => search::MultiReport, search::multi_search,
    (26214400, 8388608, 65536, 67108864, 67108864),
    r#"{"haystack":"YWJh","patterns":["YQ==","YmE=","YQ=="]}"#
    => r#"{"count":5,"complete":true,"matches":[{"pattern_id":0,"start":0,"end":1},{"pattern_id":2,"start":0,"end":1},{"pattern_id":1,"start":1,"end":3},{"pattern_id":0,"start":2,"end":3},{"pattern_id":2,"start":2,"end":3}]}"#;

    "bitap", "bitap", "Report exact Shift-And or approximate Levenshtein byte endings.",
    search::BitapRequest => search::BitapReport, search::bitap,
    (25165824, 8388608, 65536, 67108864, 8388608),
    r#"{"haystack":"YWFhYQ==","pattern":"YWE="}"#
    => r#"{"mode":"exact","matches":[{"end":2,"distance":0,"start":0},{"end":3,"distance":0,"start":1},{"end":4,"distance":0,"start":2}]}"#;

    "diff", "diff", "Generate exact edit runs or line-based Patience and Histogram diffs.",
    diff::DiffRequest => diff::DiffReport, diff::diff,
    (13631488, 8388608, 65536, 16777216, 67108864),
    r#"{"input":{"unit":"lines","left":"old\n","right":"new\n"},"unified":{}}"#
    => r#"{"method":"myers","profile":"myers_1986_delete_ties","minimal":true,"runs":[{"kind":"delete","left":{"start":0,"end":1,"byte_start":0,"byte_end":4},"right":{"start":0,"end":0,"byte_start":0,"byte_end":0}},{"kind":"insert","left":{"start":1,"end":1,"byte_start":4,"byte_end":4},"right":{"start":0,"end":1,"byte_start":0,"byte_end":4}}],"unified":"--- a/input\n+++ b/input\n@@ -1 +1 @@\n-old\n+new\n"}"#;
}
