use cube_core::operations::execution::{self, Budget};
use serde_json::json;

fn compare(operation: &str, request: serde_json::Value) -> serde_json::Value {
    execution::execute(
        operation,
        1,
        &json!({"request": request}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()["report"]
        .clone()
}

#[test]
fn hamming_counts_exact_packed_bits_and_unicode_scalars() {
    let bits = compare(
        "sequence.hamming",
        json!({"unit":"bits","left":"/wA=","right":"AIA="}),
    );
    assert_eq!(
        bits,
        json!({"differing_units":9,"total_units":16,"normalized":0.5625})
    );
    let scalars = compare(
        "sequence.hamming",
        json!({"unit":"scalars","left":"é🧊","right":"e🧊"}),
    );
    assert_eq!(scalars["differing_units"], 1);
    assert_eq!(scalars["total_units"], 2);
}

#[test]
fn unrestricted_transpositions_and_osa_are_distinct_profiles() {
    for (method, expected) in [
        ("levenshtein", 3),
        ("osa", 3),
        ("dl", 2),
        ("myers_bitvector", 3),
    ] {
        let result = compare(
            "sequence.edit_distance",
            json!({
                "operands":{"unit":"scalars","left":"CA","right":"ABC"}, "method":method
            }),
        );
        assert_eq!(result["distance"], expected);
    }
    for (threshold, distance) in [(2, json!(null)), (3, json!(3))] {
        let result = compare(
            "sequence.edit_distance",
            json!({
                "operands":{"unit":"scalars","left":"kitten","right":"sitting"},
                "method":"ukkonen","max_distance":threshold
            }),
        );
        assert_eq!(result["distance"], distance);
        assert_eq!(result["above_threshold"], threshold == 2);
    }
}
