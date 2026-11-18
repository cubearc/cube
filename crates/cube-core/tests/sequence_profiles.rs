use cube_core::{encoding::base64_encode, sequence as s};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn parse<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}
fn report(id: &str, request: Value) -> Value {
    s::invoke(id, request).unwrap()
}
fn words() -> Vec<Vec<u8>> {
    (0..=5)
        .flat_map(|n| {
            (0..1 << n).map(move |mask| (0..n).map(|i| b'a' + ((mask >> i) & 1) as u8).collect())
        })
        .collect()
}

// Independent full-row oracle. Substitution at cost two models insert/delete.
fn distance(a: &[u8], b: &[u8], substitution: usize) -> usize {
    let mut rows = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, entry) in rows[0].iter_mut().enumerate() {
        *entry = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            rows[i][j] = (rows[i - 1][j] + 1).min(rows[i][j - 1] + 1).min(
                rows[i - 1][j - 1]
                    + if a[i - 1] == b[j - 1] {
                        0
                    } else {
                        substitution
                    },
            );
        }
    }
    rows[a.len()][b.len()]
}

#[test]
fn bit_parallel_and_banded_edit_profiles_match_an_independent_dp() {
    for left in words() {
        for right in words() {
            let operands = s::Pair::Bytes {
                left: base64_encode(&left),
                right: base64_encode(&right),
            };
            let expected = distance(&left, &right, 1) as u64;
            if !left.is_empty() {
                let result = s::edit_distance(&s::EditRequest {
                    operands: operands.clone(),
                    method: s::EditMethod::MyersBitvector,
                    max_distance: None,
                })
                .unwrap();
                assert_eq!(result.distance, Some(expected), "{left:?} {right:?}");
            }
            for threshold in 0..=2 {
                let result = s::edit_distance(&s::EditRequest {
                    operands: operands.clone(),
                    method: s::EditMethod::Ukkonen,
                    max_distance: Some(threshold),
                })
                .unwrap();
                assert_eq!(
                    result.distance,
                    (expected <= u64::from(threshold)).then_some(expected)
                );
            }
        }
    }
}

#[test]
fn byte_search_profiles_match_exact_ranges_and_independent_approximate_endings() {
    for haystack in words() {
        for pattern in words()
            .into_iter()
            .filter(|word| !word.is_empty() && word.len() <= 3)
        {
            let expected: Vec<_> = haystack
                .windows(pattern.len())
                .enumerate()
                .filter_map(|(start, w)| (w == pattern).then_some((start, start + w.len())))
                .collect();
            for algorithm in [
                s::search::ExactAlgorithm::Kmp,
                s::search::ExactAlgorithm::BoyerMoore,
                s::search::ExactAlgorithm::RabinKarp,
            ] {
                let result = s::search::exact_search(&s::search::ExactRequest {
                    haystack: base64_encode(&haystack),
                    pattern: base64_encode(&pattern),
                    algorithm,
                    max_results: 65536,
                })
                .unwrap();
                assert_eq!(
                    result
                        .matches
                        .iter()
                        .map(|m| (m.start, m.end))
                        .collect::<Vec<_>>(),
                    expected
                );
            }
            let max_errors = pattern.len().min(2) - 1;
            let expected: Vec<_> = (1..=haystack.len())
                .filter_map(|end| {
                    let (cost, start) = (0..end)
                        .map(|start| (distance(&pattern, &haystack[start..end], 1), start))
                        .min()
                        .unwrap();
                    (cost <= max_errors).then_some((end, cost, Some(start)))
                })
                .collect();
            let observed = s::search::bitap(&s::search::BitapRequest {
                haystack: base64_encode(&haystack),
                pattern: base64_encode(&pattern),
                mode: s::search::BitapMode::Levenshtein,
                max_errors: Some(max_errors),
                start_witness: true,
            })
            .unwrap();
            assert_eq!(
                observed
                    .matches
                    .iter()
                    .map(|m| (m.end, m.distance, m.start))
                    .collect::<Vec<_>>(),
                expected,
                "haystack={haystack:?}, pattern={pattern:?}"
            );
        }
    }
}

#[test]
fn diff_scripts_reconstruct_and_match_the_minimum_insert_delete_cost() {
    for left in words() {
        for right in words() {
            let result = s::diff::diff(&s::diff::DiffRequest {
                input: s::diff::Input::Bytes {
                    left: base64_encode(&left),
                    right: base64_encode(&right),
                },
                method: s::diff::Method::Myers,
                occurrence_cap: None,
                unified: None,
            })
            .unwrap();
            let (mut a, mut b, mut cost) = (0, 0, 0);
            let mut rebuilt = Vec::new();
            for run in result.runs {
                assert_eq!((a, b), (run.left.start, run.right.start));
                match run.kind {
                    s::EditKind::Equal => {
                        assert_eq!(&left[a..run.left.end], &right[b..run.right.end]);
                        rebuilt.extend_from_slice(&left[a..run.left.end]);
                    }
                    s::EditKind::Insert => {
                        cost += run.right.end - b;
                        rebuilt.extend_from_slice(&right[b..run.right.end]);
                    }
                    s::EditKind::Delete => cost += run.left.end - a,
                    s::EditKind::Substitute => panic!("unexpected substitution"),
                }
                (a, b) = (run.left.end, run.right.end);
            }
            assert_eq!((a, b), (left.len(), right.len()));
            assert_eq!(rebuilt, right);
            assert_eq!(cost, distance(&left, &right, 2));
        }
    }
}

#[test]
fn deterministic_alignment_paths_and_line_diffs_preserve_provenance() {
    let affine = report(
        "sequence.align",
        json!({
            "operands":{"unit":"scalars","left":"ACCCG","right":"AG"},"mode":"global",
            "gap":{"kind":"affine","open":-3,"extend":-1}
        }),
    );
    assert_eq!(affine["score"], -1);
    assert_eq!(affine["runs"][1]["kind"], "delete");
    assert_eq!(affine["runs"][1]["left"]["start"], 1);
    assert_eq!(affine["runs"][1]["left"]["end"], 4);
    let local = report(
        "sequence.align",
        json!({
            "operands":{"unit":"scalars","left":"🧊ACZ","right":"YACY"},"mode":"local"
        }),
    );
    assert_eq!(local["score"], 4);
    assert_eq!(local["left"]["byte_start"], 4);
    assert_eq!(local["left"]["byte_end"], 6);
    let repeated = report(
        "sequence.dtw",
        json!({"left":[0,0,1],"right":[0,1],"radius":1}),
    );
    assert_eq!(repeated["path"], json!([[0, 0], [1, 0], [2, 1]]));
    assert_eq!(repeated["cost"], 0.0);
    assert!(s::dtw(&parse(json!({"left":[1,2],"right":[1],"radius":0}))).is_err());
    for method in ["myers", "patience", "histogram"] {
        let edited = report(
            "sequence.diff",
            json!({
                "input":{"unit":"lines","left":"head\r\nold","right":"head\r\nnew"},
                "method":method,"unified":{"context":0}
            }),
        );
        assert_eq!(edited["minimal"], method == "myers");
        assert_eq!(
            edited["unified"],
            "--- a/input\n+++ b/input\n@@ -2 +2 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n"
        );
    }
}

#[test]
fn metrics_cover_thresholds_exact_fractions_zero_vectors_and_model_identity() {
    let winkler = report(
        "sequence.jaro",
        json!({"left":"MARTHA","right":"MARHTA","method":"jaro_winkler"}),
    );
    assert!((winkler["similarity"].as_f64().unwrap() - 0.9611111111111111).abs() < 1e-12);
    let terms = report(
        "sequence.set_similarity",
        json!({
            "input":{"kind":"qgrams","left":"aaaa","right":"aa","q":2},"method":"qgram_l1"
        }),
    );
    assert_eq!(terms["l1"], 2);
    let js = report(
        "sequence.vector_similarity",
        json!({
            "input":{"kind":"dense","left":[0,1e150],"right":[1e-300,0]},"method":"js"
        }),
    );
    assert!((js["value"].as_f64().unwrap() - 1.0).abs() < 1e-12);
    let kl = report(
        "sequence.vector_similarity",
        json!({
            "input":{"kind":"dense","left":[1.0,0],"right":[1e-300,1e150]},"method":"kl"
        }),
    );
    assert_eq!(kl["status"], "finite");
    assert!((kl["value"].as_f64().unwrap() - 450.0 * 10.0_f64.log2()).abs() < 1e-10);
    let mut model = report(
        "sequence.tfidf_model",
        json!({"documents":[["a","b"],["a"]]}),
    );
    let score = report(
        "sequence.tfidf_compare",
        json!({"model":model,"left":["unknown"],"right":[]}),
    );
    assert_eq!(score["reason"], "zero_vector");
    model["frequencies"][0]["df"] = json!(1);
    assert!(
        s::invoke(
            "sequence.tfidf_compare",
            json!({"model":model,"left":[],"right":[]})
        )
        .is_err()
    );
}

#[test]
fn complete_search_counts_and_truncated_listings_are_distinct() {
    let result = report(
        "sequence.multi_search",
        json!({
            "haystack":"YWJh","patterns":["YQ==","YmE=","YQ=="],"max_results":1
        }),
    );
    assert_eq!(result["count"], 5);
    assert_eq!(result["complete"], false);
    assert_eq!(
        result["matches"],
        json!([{"pattern_id":0,"start":0,"end":1}])
    );
    let nonoverlap = report(
        "sequence.multi_search",
        json!({
            "haystack":"YWJh","patterns":["YQ==","YmE=","YQ=="],"overlaps":false
        }),
    );
    assert_eq!(
        nonoverlap["matches"],
        json!([
            {"pattern_id":0,"start":0,"end":1},{"pattern_id":1,"start":1,"end":3}
        ])
    );
    let empty = report(
        "sequence.hamming",
        json!({"unit":"scalars","left":"","right":""}),
    );
    assert_eq!(empty["normalized"], Value::Null);
    assert!(
        s::invoke(
            "sequence.hamming",
            json!({"unit":"bits","left":"YQ==","right":""})
        )
        .is_err()
    );
}

#[test]
fn affine_gap_parameters_allow_large_nonpositive_scores_and_reject_overflow() {
    let bounds = json!({"operands":{"unit":"scalars","left":"A","right":""},
        "mode":"global","gap":{"kind":"affine","open":-2_000_000,"extend":0}});
    assert_eq!(report("sequence.align", bounds)["score"], -2_000_000);
    let overflow = json!({"operands":{"unit":"scalars","left":"AA","right":""},
        "mode":"global","gap":{"kind":"affine","open":i64::MIN,"extend":-1}});
    assert!(s::invoke("sequence.align", overflow).is_err());
}

#[test]
fn anchored_line_strategies_choose_their_own_witnesses() {
    let input = json!({"unit":"lines","left":"a\nx\nb\nx\nc\n","right":"c\nx\nb\nx\na\n"});
    let patience = report("sequence.diff", json!({"input":input,"method":"patience"}));
    let histogram = report("sequence.diff", json!({"input":input,"method":"histogram"}));
    let first_keep = |value: &Value| {
        value["runs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|run| run["kind"] == "equal")
            .cloned()
            .unwrap()
    };
    let patience_keep = first_keep(&patience);
    assert_eq!(patience_keep["left"]["start"], 0);
    assert_eq!(patience_keep["right"]["start"], 4);
    let histogram_keep = first_keep(&histogram);
    assert_eq!(histogram_keep["left"]["start"], 1);
    assert_eq!(histogram_keep["left"]["end"], 4);
}

#[test]
fn matrix_and_pattern_limits_hold_at_the_edge() {
    let accepted = s::EditRequest {
        operands: s::Pair::Scalars {
            left: "a".repeat(2048),
            right: "a".repeat(2048),
        },
        method: s::EditMethod::Levenshtein,
        max_distance: None,
    };
    assert_eq!(s::edit_distance(&accepted).unwrap().distance, Some(0));
    let rejected = s::EditRequest {
        operands: s::Pair::Scalars {
            left: "a".repeat(2049),
            right: "a".repeat(2048),
        },
        ..accepted
    };
    assert!(s::edit_distance(&rejected).is_err());
    let bitap = s::search::BitapRequest {
        haystack: base64_encode(&[0xff; 64]),
        pattern: base64_encode(&[0xff; 63]),
        mode: s::search::BitapMode::Exact,
        max_errors: None,
        start_witness: true,
    };
    assert_eq!(s::search::bitap(&bitap).unwrap().matches.len(), 2);
    assert!(
        s::search::bitap(&s::search::BitapRequest {
            pattern: base64_encode(&[0xff; 64]),
            ..bitap
        })
        .is_err()
    );
}

#[test]
fn contiguous_tie_selection_is_directional_and_scalar_offsets_are_exact() {
    let forward = report(
        "sequence.ratcliff_obershelp",
        json!({"unit":"scalars","left":"tide","right":"diet"}),
    );
    let reverse = report(
        "sequence.ratcliff_obershelp",
        json!({"unit":"scalars","left":"diet","right":"tide"}),
    );
    assert_eq!(forward["similarity"], 0.25);
    assert_eq!(reverse["similarity"], 0.5);
    let lcs = report(
        "sequence.lcs",
        json!({"unit":"scalars","left":"xé","right":"é"}),
    );
    assert_eq!(lcs["positions"][0]["left"]["start"], 1);
    assert_eq!(lcs["positions"][0]["left"]["byte_end"], 3);
}
