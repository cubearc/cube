use cube_core::{
    errors::ErrorCode,
    pipeline::{self, Kind, Limits, Parameter, Plan, Step, Value, json, recipe::Recipe},
    text::transform,
};
use serde_json::{Value as Json, json as j};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
fn input(text: &str) -> Value {
    Value::Text { data: text.into() }
}
fn definition() -> Json {
    j!({"operation":"text.transform.masked","version":1,"parameters":{
        "selector":"fibonacci","exclude":{"unsigned_list":[13,21]},"index_base":1,
        "selected":"atbash","otherwise":"caesar","otherwise_shift":3}})
}
fn steps(def: Json) -> Vec<Step> {
    json::steps(&j!([def]).to_string()).unwrap()
}
fn plan(def: Json, limits: Limits) -> Plan {
    Plan::validate(Kind::Text, steps(def), limits).unwrap()
}
fn recipe(def: Json) -> Recipe {
    Recipe::parse(
        &j!({"format_version":1,"engine_api":1,
        "input":{"source":"slot","name":"text","type":"text"}, "output":{"type":"text"},
        "steps":[def]})
        .to_string(),
    )
    .unwrap()
}
#[test]
fn known_answer_audit_is_identical_to_direct_api_and_survives_composition() {
    let original = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let plan = plan(definition(), Limits::default());
    let result = plan.execute(input(original), &[0], None).unwrap();
    let report = &result.masked_audits[&0];
    assert_eq!(report.selected_positions, [1, 2, 3, 5, 8]);
    assert_eq!(report.deterministic_text, "ZYXGVIJSLMNOPQRSTUVWXYZABC");
    assert_eq!(report.final_text.as_bytes()[12], b'P');
    assert_eq!(report.final_text.as_bytes()[20], b'X');
    assert_eq!(result.output, input(&report.final_text));
    assert_eq!(result.intermediates[&0], result.output);
    assert_eq!(
        report,
        &transform::masked(original, &report.config).unwrap()
    );
    let repeated = plan.execute(input(original), &[], None).unwrap();
    assert_eq!(repeated.masked_audits, result.masked_audits);
    assert!(repeated.intermediates.is_empty());
    let mut chain = steps(definition());
    chain.push(Step::new("cipher.rot13"));
    let composed = Plan::validate(Kind::Text, chain, Limits::default())
        .unwrap()
        .execute(input(original), &[], None)
        .unwrap();
    assert_eq!(
        composed.output,
        input(&cube_core::ciphers::rot13(&report.final_text))
    );
    assert_eq!(composed.masked_audits[&0], *report);
    let recipe = recipe(definition());
    let bindings = BTreeMap::from([("text".into(), input(original))]);
    assert_eq!(
        recipe.execute(&bindings, &[], None).unwrap().masked_audits[&0],
        *report
    );
    assert_eq!(
        recipe
            .check(&bindings, &result.output)
            .unwrap()
            .masked_audits[&0],
        *report
    );
    assert_eq!(recipe.info().plan.engine_api, 1);
}
#[test]
fn repeated_overrides_are_retained_without_capture_and_check_returns_the_audit() {
    let mut def = definition();
    def["parameters"]["overrides"] = j!({"text_list":["1=雪", "1=🧊"]});
    let result = plan(def.clone(), Limits::default())
        .execute(input("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), &[], None)
        .unwrap();
    let report = &result.masked_audits[&0];
    assert_eq!(report.final_text, "🧊YXGVIJSLMNOPQRSTUVWXYZABC");
    assert_eq!(report.deterministic_text, "ZYXGVIJSLMNOPQRSTUVWXYZABC");
    assert_eq!(report.overrides.len(), 2);
    assert_eq!(report.overrides[0].previous, 'Z');
    assert_eq!(report.overrides[1].previous, '雪');
    let check = recipe(def)
        .check(
            &BTreeMap::from([("text".into(), input("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))]),
            &result.output,
        )
        .unwrap();
    assert_eq!(check.masked_audits[&0], *report);
}
#[test]
fn selector_types_empty_lists_unicode_and_signed_shifts_match_direct_execution() {
    for params in [
        j!({"selector":"indices","indices":{"unsigned_list":[]},"selected":"identity","overrides":{"text_list":[]}}),
        j!({"selector":"indices","indices":{"unsigned_list":[4,0,0,2]},"index_base":0,"selected":"rot13"}),
        j!({"selector":"every_nth","nth":2,"index_base":1,"selected":"atbash","otherwise":"caesar","otherwise_shift":-2147483648}),
        j!({"selector":"fibonacci","exclude":{"unsigned_list":[0,3,21]},"selected":"caesar","selected_shift":2147483647}),
    ] {
        for text in ["A !éB e\u{301}Z\0\r\n", "AaZz!"] {
            let def = j!({"operation":"text.transform.masked","version":1,"parameters":params});
            let result = plan(def, Limits::default())
                .execute(input(text), &[], None)
                .unwrap();
            let report = &result.masked_audits[&0];
            assert_eq!(report, &transform::masked(text, &report.config).unwrap());
        }
    }
    let empty = plan(definition(), Limits::default())
        .execute(input(""), &[], None)
        .unwrap();
    assert_eq!(empty.output, input(""));
    assert!(empty.masked_audits[&0].selected_positions.is_empty());
}
#[test]
fn tagged_lists_have_unambiguous_empty_types_and_exact_unsigned_transport() {
    for value in [
        j!({"unsigned_list":[]}),
        j!({"text_list":[]}),
        j!({"unsigned_list":[0,u64::MAX]}),
        j!({"text_list":["1=🧊"]}),
    ] {
        let parameter: Parameter = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parameter).unwrap(), value);
    }
    for value in [
        j!([]),
        j!([1, 2]),
        j!({"unsigned_list":[-1]}),
        j!({"unsigned_list":[1.0]}),
        j!({"unsigned_list":[true]}),
        j!({"unsigned_list":["1"]}),
        j!({"unsigned_list":[[1]]}),
        j!({"text_list":[1]}),
        j!({"text_list":[null]}),
        j!({"text_list":[],"unsigned_list":[]}),
        j!({"unsigned_list":[],"unknown":1}),
        j!(null),
    ] {
        assert!(
            serde_json::from_value::<Parameter>(value.clone()).is_err(),
            "{value}"
        );
    }
    assert!(
        serde_json::from_str::<Parameter>(r#"{"unsigned_list":[18446744073709551616]}"#).is_err()
    );
    let mut def = definition();
    def["parameters"]["exclude"] = j!({"unsigned_list":[13,21]});
    assert!(Plan::validate(Kind::Text, steps(def), Limits::default()).is_ok());
}
#[test]
fn invalid_static_contracts_fail_before_earlier_steps_and_positions_before_transform() {
    for params in [
        j!({"selector":"every_nth","nth":0,"selected":"atbash"}),
        j!({"selector":"indices","selected":"atbash"}),
        j!({"selector":"fibonacci","exclude":{"text_list":[]},"selected":"atbash"}),
        j!({"selector":"fibonacci","exclude":{"unsigned_list":[4]},"selected":"atbash"}),
        j!({"selector":"fibonacci","selected":"caesar"}),
        j!({"selector":"fibonacci","selected":"rot13","selected_shift":3}),
        j!({"selector":"fibonacci","selected":"rot13","overrides":{"text_list":["1=é"]}}),
    ] {
        let bad = j!({"operation":"text.transform.masked","version":1,"parameters":params});
        let mut list = vec![Step::new("cipher.rot13")];
        list.extend(steps(bad));
        let error = Plan::validate(Kind::Text, list, Limits::default()).unwrap_err();
        assert_eq!(error.step, Some(1));
        assert_eq!(error.operation.as_deref(), Some("text.transform.masked"));
    }
    for params in [
        j!({"selector":"indices","indices":{"unsigned_list":[u64::MAX]},"selected":"atbash"}),
        j!({"selector":"indices","indices":{"unsigned_list":[0]},"index_base":1,"selected":"atbash"}),
        j!({"selector":"fibonacci","selected":"identity","overrides":{"text_list":["99=X"]}}),
    ] {
        let def = j!({"operation":"text.transform.masked","version":1,"parameters":params});
        let parsed = Plan::validate(Kind::Text, steps(def), Limits::default());
        if usize::BITS == 32 && parsed.is_err() {
            continue;
        }
        let error = parsed
            .unwrap()
            .execute(input("ABC"), &[], None)
            .unwrap_err();
        assert_eq!(error.error.code, ErrorCode::InvalidParameter);
        assert_eq!(error.step, Some(0));
    }
    let mut bad_version = definition();
    bad_version["version"] = j!(2);
    assert!(Plan::validate(Kind::Text, steps(bad_version), Limits::default()).is_err());
    assert!(Plan::validate(Kind::Bytes, steps(definition()), Limits::default()).is_err());
}
#[test]
fn audits_and_captures_share_an_exact_preflight_budget_and_work_is_charged() {
    let original = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    // 512 report header + 3*26 text bytes + 5*8 selected positions + 2*8 excluded values.
    let audit = 646;
    for (capture, bound) in [(vec![], audit), (vec![0], audit + 26)] {
        let limits = Limits {
            max_capture_bytes: bound,
            ..Limits::default()
        };
        assert!(
            plan(definition(), limits)
                .execute(input(original), &capture, None)
                .is_ok()
        );
        let limits = Limits {
            max_capture_bytes: bound - 1,
            ..Limits::default()
        };
        let error = plan(definition(), limits)
            .execute(input(original), &capture, None)
            .unwrap_err();
        assert_eq!(error.error.code, ErrorCode::ResourceLimit);
    }
    let result = plan(definition(), Limits::default())
        .execute(input(original), &[], None)
        .unwrap();
    assert!(result.work_bytes_upper_bound > 3 * 26);
    for limit in [
        result.work_bytes_upper_bound,
        result.work_bytes_upper_bound - 1,
    ] {
        let limits = Limits {
            max_work_bytes: limit,
            ..Limits::default()
        };
        assert_eq!(
            plan(definition(), limits)
                .execute(input(original), &[], None)
                .is_ok(),
            limit == result.work_bytes_upper_bound
        );
    }
    let def = j!({"operation":"text.transform.masked","version":1,"parameters":{"selector":"every_nth","nth":1,"selected":"identity"}});
    assert_eq!(
        plan(def, Limits::default())
            .execute(input(&"A".repeat(65537)), &[], None)
            .unwrap_err()
            .error
            .code,
        ErrorCode::ResourceLimit
    );
    let token = AtomicBool::new(true);
    assert_eq!(
        plan(definition(), Limits::default())
            .execute(input(original), &[], Some(&token))
            .unwrap_err()
            .error
            .code,
        ErrorCode::Cancelled
    );
}
#[test]
fn full_request_fits_eight_bindings_and_parameter_limits_include_container_storage() {
    let mut def = definition();
    def["parameters"]["selected"] = j!("caesar");
    def["parameters"]["selected_shift"] = j!(-3);
    def["parameters"]["overrides"] = j!({"text_list":["1=X"]});
    assert_eq!(def["parameters"].as_object().unwrap().len(), 8);
    assert!(
        plan(def, Limits::default())
            .execute(input("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), &[], None)
            .is_ok()
    );
    for param in [
        Parameter::UnsignedList {
            values: vec![0; 65537],
        },
        Parameter::TextList {
            values: vec![String::new(); 65537],
        },
        Parameter::TextList {
            values: vec!["x".repeat(pipeline::MAX_PARAMETER_BYTES)],
        },
    ] {
        let mut step = Step::new("text.transform.masked");
        step.parameters.insert("overrides".into(), param);
        assert_eq!(
            Plan::validate(Kind::Text, vec![step], Limits::default())
                .unwrap_err()
                .error
                .code,
            ErrorCode::ResourceLimit
        );
    }
    let document = j!([{"operation":"text.transform.masked","version":1,"parameters":{"selector":"indices","indices":{"unsigned_list":vec![0;4096]},"selected":"identity"}}]);
    assert_eq!(
        json::steps(&document.to_string()).unwrap_err().error.code,
        ErrorCode::ResourceLimit
    );
    assert!(json::steps(&format!("{} ", " ".repeat(json::MAX_DOCUMENT_BYTES))).is_err());
    assert!(json::steps(r#"[{"operation":"text.transform.masked","version":1,"parameters":{"indices":{"unsigned_list":[],"unsigned_list":[]}}}]"#).is_err());
}
#[test]
fn legacy_execution_wire_shape_and_recipe_contracts_are_unchanged() {
    let result = Plan::validate(
        Kind::Text,
        vec![Step::new("cipher.rot13")],
        Limits::default(),
    )
    .unwrap()
    .execute(input("ABC"), &[], None)
    .unwrap();
    let serialized = serde_json::to_value(result).unwrap();
    assert_eq!(
        serialized,
        j!({"output":{"type":"text","data":"NOP"},"intermediates":{},"work_bytes_upper_bound":9})
    );
    let info = pipeline::adapter_version("text.transform.masked");
    assert_eq!(info, Some(1));
}

#[test]
fn exact_parameter_bytes_and_output_bounds_are_enforced_before_transforming() {
    let request = |extra: usize| {
        let mut step = Step::new("text.transform.masked");
        step.parameters = BTreeMap::from([
            ("selector".into(), Parameter::Text("fibonacci".into())),
            ("selected".into(), Parameter::Text("identity".into())),
            // 17 scalar bytes + 16 list header + 24 string header + record bytes.
            (
                "overrides".into(),
                Parameter::TextList {
                    values: vec![format!(
                        "{}1=X",
                        "0".repeat(pipeline::MAX_PARAMETER_BYTES - 60 + extra)
                    )],
                },
            ),
        ]);
        step
    };
    assert!(Plan::validate(Kind::Text, vec![request(0)], Limits::default()).is_ok());
    let error = Plan::validate(Kind::Text, vec![request(1)], Limits::default()).unwrap_err();
    assert_eq!(error.error.code, ErrorCode::ResourceLimit);
    assert!(error.parameter.is_some());
    let mut def = definition();
    def["parameters"]["overrides"] = j!({"text_list":["1=🧊"]});
    // A replacement can expand one scalar from one UTF-8 byte to four.
    for limit in [29, 28] {
        let limits = Limits {
            max_value_bytes: limit,
            ..Limits::default()
        };
        assert_eq!(
            plan(def.clone(), limits)
                .execute(input("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), &[], None)
                .is_ok(),
            limit == 29
        );
    }
    let mut chain = vec![
        Step::new("encoding.hex.decode"),
        Step::new("text.utf8.decode"),
    ];
    chain.extend(steps(definition()));
    let limits = Limits {
        max_capture_bytes: 0,
        ..Limits::default()
    };
    let error = Plan::validate(Kind::Text, chain, limits)
        .unwrap()
        .execute(input("zz"), &[], None)
        .unwrap_err();
    assert_eq!(error.error.code, ErrorCode::ResourceLimit);
    assert_eq!(error.step, Some(2));
}
