//! Source-qualified pin/lug profiles through the public executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn run(text: &str, settings: Value) -> Value {
    execution::execute(
        "cipher.hagelin.encode",
        1,
        &json!({"text":text,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn m209_complete_1942_manual_check_matches_all_twenty_six_letters() {
    // TM11-380, 27 April1942, pp16–18. Supplied indices are the actual
    // contacts PONMLK corresponding to windowAAAAAA, not the window indices.
    let v: Value = serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/hagelin_vectors.json"
    ))
    .unwrap();
    assert_eq!(
        run(v["text"].as_str().unwrap(), v["settings"].clone())["text"],
        v["expected"]
    );
}
fn settings(profile: &str) -> Value {
    let lengths: Vec<usize> = match profile {
        "c36_fixed25_bouch_v1" => vec![25, 23, 21, 19, 17],
        "c52_c2_27_v1" => vec![26, 25, 46, 42, 38, 34],
        "cx52_m27_late_v1" => vec![47; 6],
        _ => vec![26, 25, 23, 21, 19, 17],
    };
    let pins: Vec<String> = lengths.iter().map(|n| "0".repeat(*n)).collect();
    let mut p =
        json!({"profile":profile,"pins":pins,"initial_state":vec![0;lengths.len()],"trace":true});
    if profile != "c36_fixed25_bouch_v1" {
        p["lugs"] = json!(vec![vec![false; 6]; 27]);
    }
    if profile != "m209_standard_v1" {
        p["slide"] = json!(0);
    }
    p
}
const PROFILES: [&str; 5] = [
    "c36_fixed25_bouch_v1",
    "c38_standard27_v1",
    "m209_standard_v1",
    "c52_c2_27_v1",
    "cx52_m27_late_v1",
];
#[test]
fn fixed_c36_uses_its_own_distribution_in_all_thirty_two_selections() {
    for mask in 0..32u8 {
        let mut p = settings(PROFILES[0]);
        for wheel in 0..5 {
            if mask & (1 << wheel) != 0 {
                let n = p["pins"][wheel].as_str().unwrap().len();
                p["pins"][wheel] = json!("1".repeat(n));
            }
        }
        let kick: u8 = [1, 2, 3, 7, 12]
            .into_iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, n)| n)
            .sum();
        let result = run("A", p);
        assert_eq!(
            result["text"],
            String::from(char::from(b'A' + kick % 26)),
            "mask{mask}"
        );
        assert_eq!(result["trace"][0]["displacement"], kick);
        assert_eq!(result["final_state"], json!([1, 1, 1, 1, 1]));
    }
}
#[test]
fn overlapping_lugs_count_bars_once_and_all_slides_are_reciprocal() {
    for profile in &PROFILES[1..] {
        for slide in 0..26u8 {
            if *profile == PROFILES[2] && slide != 25 {
                continue;
            }
            let mut p = settings(profile);
            if *profile != PROFILES[2] {
                p["slide"] = json!(slide);
            }
            for wheel in 0..2 {
                let n = p["pins"][wheel].as_str().unwrap().len();
                p["pins"][wheel] = json!("1".repeat(n));
            }
            p["lugs"] = json!(vec![vec![true, true, false, false, false, false]; 27]);
            let r = run("ABCDEFGHIJKLMNOPQRSTUVWXYZ", p.clone());
            assert_eq!(r["trace"][0]["displacement"], 27);
            assert_eq!(
                r["text"].as_str().unwrap().as_bytes()[0],
                b'A' + (slide + 27) % 26
            );
            let decoded = execution::execute(
                "cipher.hagelin.decode",
                1,
                &json!({"text":r["text"],"settings":p}),
                &json!({}),
                Budget::default(),
            )
            .unwrap();
            assert_eq!(decoded["text"], "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
            assert_eq!(decoded["final_state"], r["final_state"]);
        }
    }
}
#[test]
fn later_cx52m_latches_pins_and_uses_all_sixty_four_prefix_selections() {
    for mask in 0..64u8 {
        let mut p = settings(PROFILES[4]);
        for wheel in 0..6 {
            let first = if mask & (1 << wheel) != 0 { "1" } else { "0" };
            p["pins"][wheel] = json!(format!("{first}{}", "0".repeat(46)));
        }
        p["lugs"] = json!(vec![vec![true, true, true, true, true, true]; 27]);
        let r = run("A", p);
        let state: Vec<u8> = (0..6)
            .map(|i| u8::from(i == 0 || mask & ((1 << i) - 1) != 0))
            .collect();
        assert_eq!(r["final_state"], json!(state), "mask{mask}");
        assert_eq!(r["trace"][0]["active_mask"], mask);
        // The five motion bars never add to the27 printing-bar displacement.
        assert_eq!(
            r["trace"][0]["displacement"],
            if mask == 0 { 0 } else { 27 }
        );
    }
}
#[test]
fn c52_c2_double_last_four_contacts_interoperate_with_complete_m209_manual_key() {
    let v: Value = serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/hagelin_vectors.json"
    ))
    .unwrap();
    let mut p = v["settings"].clone();
    p["profile"] = json!(PROFILES[3]);
    p["slide"] = json!(25);
    for wheel in 2..6 {
        let pins = p["pins"][wheel].as_str().unwrap();
        p["pins"][wheel] = json!(pins.chars().flat_map(|c| [c, c]).collect::<String>());
        p["initial_state"][wheel] = json!(p["initial_state"][wheel].as_u64().unwrap() * 2);
    }
    let r = run(v["text"].as_str().unwrap(), p);
    assert_eq!(r["text"], v["expected"]);
    assert_eq!(r["final_state"], json!([15, 15, 32, 34, 36, 4]));
}
#[test]
fn every_profile_has_exact_continuation_wraps_and_empty_identity() {
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for profile in PROFILES {
        let mut p = settings(profile);
        let lengths: Vec<usize> = p["pins"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().len())
            .collect();
        for (wheel, n) in lengths.iter().enumerate() {
            p["pins"][wheel] = json!(
                (0..*n)
                    .map(|i| if (i + wheel) % 3 == 0 { '1' } else { '0' })
                    .collect::<String>()
            );
            p["initial_state"][wheel] = json!(n - 1);
        }
        if profile != PROFILES[0] {
            p["lugs"][0] = json!([true, false, true, false, false, false]);
        }
        let whole = run(text, p.clone());
        let empty = run("", p.clone());
        assert_eq!(empty["text"], "");
        assert_eq!(empty["final_state"], p["initial_state"]);
        assert_eq!(empty["trace"], json!([]));
        for split in 0..=text.len() {
            let first = run(&text[..split], p.clone());
            let mut next = p.clone();
            next["initial_state"] = first["final_state"].clone();
            let second = run(&text[split..], next);
            assert_eq!(
                format!(
                    "{}{}",
                    first["text"].as_str().unwrap(),
                    second["text"].as_str().unwrap()
                ),
                whole["text"]
            );
            assert_eq!(second["final_state"], whole["final_state"]);
        }
    }
}
#[test]
fn typed_settings_reject_wrong_machine_tables_and_private_invalid_input() {
    let base = settings(PROFILES[2]);
    let changes = [
        ("profile", json!("generic")),
        ("slide", json!(0)),
        ("pins", json!(["0"])),
        ("initial_state", json!([26, 0, 0, 0, 0, 0])),
        ("secret", json!("PRIVATE_KEY")),
    ];
    for (field, value) in changes {
        let mut p = base.clone();
        p[field] = value;
        assert!(
            execution::execute(
                "cipher.hagelin.encode",
                1,
                &json!({"text":"A","settings":p}),
                &json!({}),
                Budget::default()
            )
            .is_err(),
            "{field}"
        );
    }
    for profile in PROFILES {
        let p = settings(profile);
        for text in ["a", "A A", "é", "PRIVATE_KEY"] {
            let err = execution::execute(
                "cipher.hagelin.encode",
                1,
                &json!({"text":text,"settings":p}),
                &json!({}),
                Budget::default(),
            )
            .unwrap_err();
            assert!(!format!("{err:?}").contains("PRIVATE_KEY"));
        }
    }
    // Native typed calls retain domain checks independently of schema admission.
    use cube_core::{ciphers::mechanical::hagelin, errors::ErrorCode};
    for (profile, field, value) in [
        (
            PROFILES[2],
            "pins",
            json!([
                "x".repeat(26),
                "0".repeat(25),
                "0".repeat(23),
                "0".repeat(21),
                "0".repeat(19),
                "0".repeat(17)
            ]),
        ),
        (PROFILES[1], "slide", json!(26)),
        (PROFILES[2], "initial_state", json!([26, 0, 0, 0, 0, 0])),
    ] {
        let mut p = settings(profile);
        p[field] = value;
        let p: hagelin::Settings = serde_json::from_value(p).unwrap();
        assert_eq!(
            hagelin::encode("A", &p).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
    let mut p = base;
    p["lugs"][0] = json!([true, true, true, false, false, false]);
    assert!(
        execution::execute(
            "cipher.hagelin.encode",
            1,
            &json!({"text":"A","settings":p}),
            &json!({}),
            Budget::default()
        )
        .is_err()
    );
}
#[test]
fn native_and_public_limits_fail_instead_of_truncating() {
    use cube_core::ciphers::mechanical::{MAX_INPUT, MAX_TRACE, hagelin};
    let mut p = settings(PROFILES[2]);
    p["trace"] = json!(false);
    let typed: hagelin::Settings = serde_json::from_value(p.clone()).unwrap();
    let exact = "A".repeat(MAX_INPUT);
    assert_eq!(
        hagelin::encode(&exact, &typed).unwrap().text.len(),
        MAX_INPUT
    );
    let over = format!("{exact}A");
    assert_eq!(
        hagelin::encode(&over, &typed).unwrap_err().code,
        cube_core::errors::ErrorCode::ResourceLimit
    );
    assert!(
        execution::execute(
            "cipher.hagelin.encode",
            1,
            &json!({"text":over,"settings":p}),
            &json!({}),
            Budget::default()
        )
        .is_err()
    );
    p["trace"] = json!(true);
    let traced = serde_json::from_value(p).unwrap();
    assert_eq!(
        hagelin::encode(&"A".repeat(10000), &traced)
            .unwrap()
            .trace
            .len(),
        10000
    );
    assert!(hagelin::encode(&"A".repeat(MAX_TRACE + 1), &traced).is_err());
    assert!(
        hagelin::encode(&"A".repeat(MAX_TRACE), &traced).is_err(),
        "independent full-report4MiB ceiling"
    );
}
