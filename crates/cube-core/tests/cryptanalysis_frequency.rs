use cube_core::cryptanalysis::{self as a, frequency};
use serde_json::json;
use std::sync::atomic::AtomicBool;

fn request() -> serde_json::Value {
    json!({"bins":[{"label":"A","observed":"20","expected":25.0},{"label":"B","observed":30,"expected":25.0},{"label":"C","observed":50,"expected":50.0}],"expected_policy":"counts"})
}
fn compare(value: serde_json::Value) -> frequency::Report {
    frequency::compare(&a::parse_request(&value.to_string()).unwrap(), None).unwrap()
}

#[test]
fn hand_computed_distance_and_explicit_normalization_agree() {
    let r = compare(request());
    assert_eq!(r.distance, frequency::Distance::Finite { value: 2.0 });
    assert_eq!(r.degrees_of_freedom, Some(2));
    assert_eq!(r.observed_total, 100);
    assert_eq!(r.status.work, 100);
    assert!(r.status.complete);
    assert_eq!(
        r.bins[0].distance,
        frequency::Distance::Finite { value: 1.0 }
    );
    let mut v = request();
    v["expected_policy"] = json!("normalize_proportions");
    for (bin, weight) in v["bins"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip([1.0, 1.0, 2.0])
    {
        bin["expected"] = json!(weight);
    }
    let normalized = compare(v.clone());
    assert_eq!(normalized.distance, r.distance);
    assert_eq!(normalized.bins[0].expected, 25.0);
    assert_ne!(normalized.model_sha256, r.model_sha256);
    assert_eq!(normalized.model_sha256, compare(v).model_sha256);
}

#[test]
fn zero_mass_empty_input_and_overflow_are_explicit() {
    let mut v = request();
    v["bins"][0]["expected"] = json!(0.0);
    v["bins"][1]["expected"] = json!(50.0);
    assert_eq!(
        compare(v).distance,
        frequency::Distance::ZeroExpectedMismatch
    );
    let r = compare(
        json!({"bins":[{"label":"none","observed":0,"expected":0.0}],"expected_policy":"counts"}),
    );
    assert_eq!(r.distance, frequency::Distance::Undefined);
    assert_eq!(r.bins[0].distance, frequency::Distance::Undefined);
    assert_eq!(r.degrees_of_freedom, None);
    let r = compare(
        json!({"bins":[{"label":"rare","observed":16777216,"expected":1e-300},{"label":"common","observed":0,"expected":1e12}],"expected_policy":"normalize_proportions","limits":{"max_work":16777216}}),
    );
    assert_eq!(r.distance, frequency::Distance::OutOfRange);
    assert!(!r.status.complete);
}

#[test]
fn invalid_models_and_exact_work_limits_are_not_silent() {
    for value in [
        json!({"bins":[{"label":"A","observed":2,"expected":1.0}],"expected_policy":"counts"}),
        json!({"bins":[{"label":"A","observed":1,"expected":-1.0}],"expected_policy":"counts"}),
        json!({"bins":[{"label":"A","observed":0,"expected":0.0}],"expected_policy":"normalize_proportions"}),
        json!({"bins":[{"label":"A","observed":0,"expected":0.0},{"label":"A","observed":0,"expected":0.0}],"expected_policy":"counts"}),
        json!({"bins":[{"label":"A","observed":16777217,"expected":16777217.0}],"expected_policy":"counts"}),
    ] {
        let typed = a::parse_request(&value.to_string()).unwrap();
        assert!(frequency::compare(&typed, None).is_err());
    }
    let mut value = request();
    value["limits"] = json!({"max_work":100});
    assert!(compare(value.clone()).status.complete);
    value["limits"]["max_work"] = json!(99);
    let r = compare(value.clone());
    assert_eq!(r.status.stop, a::Stop::WorkLimit);
    assert_eq!(r.bins.len(), 2);
    assert_eq!(r.degrees_of_freedom, None);
    value["limits"] = json!({"max_results":1});
    assert_eq!(compare(value).status.stop, a::Stop::ResultLimit);
    let cancel = AtomicBool::new(true);
    let r = frequency::compare(
        &a::parse_request(&request().to_string()).unwrap(),
        Some(&cancel),
    )
    .unwrap();
    assert_eq!(r.status.stop, a::Stop::Cancelled);
    assert!(r.bins.is_empty());
}

#[test]
fn structural_limits_reject_one_over_and_preserve_exact_boundaries() {
    let mut value = json!({"bins":[{"label":"A".repeat(256),"observed":16777216,"expected":16777216.0}],"expected_policy":"counts"});
    assert!(compare(value.clone()).status.complete);
    value["bins"][0]["label"] = json!("A".repeat(257));
    let typed = a::parse_request(&value.to_string()).unwrap();
    assert!(frequency::compare(&typed, None).is_err());
    value = json!({"bins":(0..10000).map(|n| json!({"label":n.to_string(),"observed":0,"expected":0.0})).collect::<Vec<_>>(),"expected_policy":"counts"});
    let report = compare(value.clone());
    assert_eq!(report.bins.len(), 4096);
    assert_eq!(report.status.stop, a::Stop::ResultLimit);
    value["bins"]
        .as_array_mut()
        .unwrap()
        .push(json!({"label":"one-over","observed":0,"expected":0.0}));
    let typed = a::parse_request(&value.to_string()).unwrap();
    assert!(frequency::compare(&typed, None).is_err());
}
