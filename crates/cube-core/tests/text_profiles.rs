use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};

fn run(operation: &str, request: Value) -> Result<Value, execution::Failure> {
    execution::execute(
        operation,
        1,
        &json!({"request": request}),
        &json!({}),
        Budget {
            input_bytes: cube_core::text::profiles::MAX_DOCUMENT,
            output_bytes: 9 << 20,
            ..Budget::default()
        },
    )
    .map(|mut outputs| outputs["report"].take())
}

#[test]
fn ragged_interleave_round_trips_exact_scalars() {
    let result = run(
        "text.scalar_weave",
        json!({"action":"interleave","mode":"ragged","lanes":["Aé","B","","🧊C"]}),
    )
    .unwrap();
    assert_eq!(
        result,
        json!({"action":"interleave","text":"AB🧊éC","lengths":[2,1,0,2]})
    );
    let reversed = run(
        "text.scalar_weave",
        json!({"action":"deinterleave","text":result["text"],"lengths":result["lengths"]}),
    )
    .unwrap();
    assert_eq!(
        reversed,
        json!({"action":"deinterleave","lanes":["Aé","B","","🧊C"]})
    );
}

#[test]
fn whole_words_preserve_duplicates_and_original_offsets() {
    let result = run(
        "text.select_words",
        json!({"text":"é!\u{2003}B","indices":[-1,1,1,0],"out_of_range":"skip"}),
    )
    .unwrap();
    assert_eq!(
        result["selected"],
        json!([
            {"index":-1,"word_index":1,"text":"B","byte_start":6,"byte_end":7,"scalar_start":3,"scalar_end":4},
            {"index":1,"word_index":0,"text":"é!","byte_start":0,"byte_end":3,"scalar_start":0,"scalar_end":2},
            {"index":1,"word_index":0,"text":"é!","byte_start":0,"byte_end":3,"scalar_start":0,"scalar_end":2}
        ])
    );
    assert_eq!(result["omitted"], json!([{"position":3,"index":0}]));
}

#[test]
fn unicode_folding_and_grapheme_boundaries_remain_explicit() {
    let folded = run(
        "text.unicode_transform",
        json!({"transform":"casefold","text":"Fuß İΣ"}),
    )
    .unwrap();
    assert_eq!(folded["text"], "fuss i̇σ");
    let turkic = run(
        "text.unicode_transform",
        json!({"transform":"casefold","profile":"turkic","text":"Iİıi\u{0}"}),
    )
    .unwrap();
    assert_eq!(turkic["text"], "ıiıi\u{0}");
    let simple = run(
        "text.unicode_transform",
        json!({"transform":"casefold","profile":"simple","text":"ßẞİ"}),
    )
    .unwrap();
    assert_eq!(simple["text"], "ßßİ");
    let filtered = run(
        "text.unicode_transform",
        json!({"transform":"category_filter","text":"AⅧä🥝́ 2","categories":["Lu","Mn"]}),
    )
    .unwrap();
    assert_eq!(filtered["text"], "Á");
    let out = run(
        "text.graphemes",
        json!({"action":"segments","text":"á🇧🇷\r\n"}),
    )
    .unwrap();
    assert_eq!(
        out["segments"],
        json!([
            {"byte_start":0,"byte_end":3,"scalar_start":0,"scalar_end":2},
            {"byte_start":3,"byte_end":11,"scalar_start":2,"scalar_end":4},
            {"byte_start":11,"byte_end":13,"scalar_start":4,"scalar_end":6}
        ])
    );
    let reversed = run("text.graphemes", json!({"action":"reverse","text":"á🧑‍💻b"})).unwrap();
    assert_eq!(reversed["text"], "b🧑‍💻á");
}

#[test]
fn regex_keeps_empty_captures_separate_from_missing_groups() {
    let out = run(
        "text.regex",
        json!({"action":"captures","text":"é","pattern":"(?<first>é)(b)?(c*)","max_results":4}),
    )
    .unwrap();
    assert_eq!(out["matches"][0]["matched"]["text"], "é");
    assert_eq!(out["matches"][0]["groups"][0]["name"], "first");
    assert!(out["matches"][0]["groups"][1]["matched"].is_null());
    assert_eq!(
        out["matches"][0]["groups"][2]["matched"],
        json!({"text":"","byte_start":2,"byte_end":2,"scalar_start":1,"scalar_end":1})
    );
    let out = run(
        "text.regex",
        json!({"action":"split","text":"é界","pattern":"","max_results":8,"keep_empty":true}),
    )
    .unwrap();
    assert_eq!(
        out["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["text"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["", "é", "界", ""]
    );
    assert!(
        run(
            "text.regex",
            json!({"action":"matches","text":"aaa","pattern":"a","max_results":2})
        )
        .is_err()
    );
    assert!(
        run(
            "text.regex",
            json!({"action":"matches","text":"ab","pattern":"(a)\\1"})
        )
        .is_err()
    );
}

#[test]
fn established_phonetic_known_answers_call_core() {
    let enc = run(
        "text.phonetic",
        json!({"method":"soundex_nara","names":["Washington","Lee","Ashcraft"]}),
    )
    .unwrap();
    assert_eq!(
        enc["keys"],
        json!([{"key":"W252"},{"key":"L000"},{"key":"A261"}])
    );
    let m = run(
        "text.phonetic",
        json!({"method":"double_metaphone","names":["Smith","Schmidt","B"]}),
    )
    .unwrap();
    assert_eq!(
        m["keys"],
        json!([{"primary":"SM0","alternate":"XMT"},{"primary":"XMT","alternate":"SMT"},{"primary":"P","alternate":"P"}])
    );
    let v = run(
        "text.phonetic",
        json!({"method":"cologne","name":"MüllerLüdenscheidt"}),
    )
    .unwrap();
    assert_eq!(v["keys"][0]["key"], "65752682");
    let dm = run("text.daitch_mokotoff", json!({"name":"Auerbach"})).unwrap();
    assert_eq!(dm["codes"], json!(["097400", "097500"]));
    let mra = run(
        "text.match_rating",
        json!({"action":"compare","left":"a","right":"A"}),
    )
    .unwrap();
    assert_eq!(mra["matches"], true);
}

#[test]
fn beider_morse_keeps_explicit_languages_and_ordered_prefix_alternatives() {
    let result = run("text.beider_morse",json!({"name":"d'Angelo","name_type":"generic","rule_type":"exact","language":{"mode":"explicit","languages":["italian"]}})).unwrap();
    let groups = result["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["path"], json!([{"kind":"prefix_separated"}]));
    assert_eq!(groups[1]["path"], json!([{"kind":"prefix_joined"}]));
    assert!(
        groups
            .iter()
            .all(|group| group["languages"] == json!(["italian"]))
    );
    assert!(groups.iter().all(|group| {
        group["alternatives"]
            .as_array()
            .unwrap()
            .windows(2)
            .all(|pair| pair[0].as_str().unwrap() < pair[1].as_str().unwrap())
    }));
    let unsupported = run(
        "text.beider_morse",
        json!({"name":"Smith","name_type":"sephardic","rule_type":"exact","language":{"mode":"explicit","languages":["turkish"]}}),
    ).unwrap_err();
    assert_eq!(unsupported.0, "unsupported_format");
}

#[test]
fn stable_ascii_collation_preserves_ties_even_in_descending_order() {
    let result = run("text.sort", json!({
        "unit":"tokens","tokens":["A","a","B","b"],"order":"descending","collation":"ascii_letters"
    })).unwrap();
    assert_eq!(result["tokens"], json!(["B", "b", "A", "a"]));
    assert_eq!(result["original_indices"], json!([2, 3, 0, 1]));
    let result = run("text.sort", json!({"unit":"scalars","text":"aAäá"})).unwrap();
    assert_eq!(result["text"], "Aaaä́");
    assert_eq!(result["original_indices"], json!([1, 0, 3, 2, 4]));
}

#[test]
fn scalar_width_and_source_and_record_caps_reject_one_over() {
    let width = 65_536;
    let exact = run(
        "text.scalar_weave",
        json!({"action":"pad","text":"","pad":"🧊","width":width}),
    )
    .unwrap();
    assert_eq!(exact["text"].as_str().unwrap().len(), 4 * width);
    assert!(
        run(
            "text.scalar_weave",
            json!({"action":"pad","text":"","width":width+1})
        )
        .is_err()
    );
    assert!(
        run(
            "text.scalar_weave",
            json!({"action":"deinterleave","text":"it","lengths":[1,0]})
        )
        .is_err()
    );
    assert!(
        run(
            "text.scalar_weave",
            json!({"action":"interleave","lanes":["A",""],"mode":"equal_lengths"})
        )
        .is_err()
    );

    let max = 1 << 20;
    let cluster = format!("e{}", "\u{301}".repeat(max / 2 - 1));
    assert!(
        run(
            "text.graphemes",
            json!({"action":"segments","text":format!("{cluster}a")})
        )
        .is_ok()
    );
    assert!(
        run(
            "text.graphemes",
            json!({"action":"segments","text":format!("{cluster}aa")})
        )
        .is_err()
    );
    assert!(
        run(
            "text.graphemes",
            json!({"action":"segments","text":"a".repeat(width+1)})
        )
        .is_err()
    );
    assert!(
        run(
            "text.unicode_transform",
            json!({"transform":"ascii_upper","text":"n".repeat(max)})
        )
        .is_ok()
    );
    assert!(
        run(
            "text.unicode_transform",
            json!({"transform":"ascii_upper","text":"n".repeat(max+1)})
        )
        .is_err()
    );
    assert!(
        run(
            "text.unicode_transform",
            json!({"transform":"category_filter","text":"Å","categories":["Lu","Lu"]})
        )
        .is_err()
    );
    assert!(
        run(
            "text.phonetic",
            json!({"method":"nysiis","names":vec!["A".repeat(256);256]})
        )
        .is_ok()
    );
    assert!(
        run(
            "text.phonetic",
            json!({"method":"nysiis","name":"a".repeat(257)})
        )
        .is_err()
    );
}

#[test]
fn lexical_comparisons_and_regex_construction_stay_within_work_budgets() {
    let near = vec!["a".repeat(14); 65_536];
    assert!(run("text.sort", json!({"unit":"tokens","tokens":near})).is_ok());
    let too_expensive = vec!["a".repeat(16); 65_536];
    assert!(run("text.sort", json!({"unit":"tokens","tokens":too_expensive})).is_err());
    for pattern in [
        "a".repeat(4097),
        "(?:a{9999}){9999}".to_string(),
        format!("{}a{}", "(?:".repeat(65), ")".repeat(65)),
        "()".repeat(65),
    ] {
        assert!(
            run(
                "text.regex",
                json!({"action":"exists","text":"a","pattern":pattern})
            )
            .is_err()
        );
    }
    assert!(
        run(
            "text.regex",
            json!({"action":"exists","text":"","pattern":"a".repeat(4096)})
        )
        .is_ok()
    );
}

#[test]
fn cologne_keeps_the_previous_code_even_when_it_does_not_emit_it() {
    let report = run(
        "text.phonetic",
        json!({"method":"cologne","names":["Ashcraft","Weiß","Äüöß"]}),
    )
    .unwrap();
    assert_eq!(
        report["keys"],
        json!([{"key":"088732"},{"key":"38"},{"key":"08"}])
    );
}
