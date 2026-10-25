use cube_core::pipeline::{self, ForensicReport, Kind, Limits, Parameter, Plan, Step, Value};
fn step(id: &str, params: &[(&str, i64)]) -> Step {
    let mut s = Step::new(id);
    s.parameters = params
        .iter()
        .map(|(k, v)| ((*k).into(), Parameter::Integer(*v)))
        .collect();
    s
}
fn run(data: &[u8], steps: Vec<Step>, capture: &[usize]) -> pipeline::Execution {
    Plan::validate(Kind::Bytes, steps, Limits::default())
        .unwrap()
        .execute(
            Value::Bytes {
                data: data.to_vec(),
            },
            capture,
            None,
        )
        .unwrap()
}
#[test]
fn forensic_results_preserve_bytes_offsets_and_captures() {
    let result = run(
        b"A\0\xffB",
        vec![step(
            "file.hex.inspect",
            &[("offset", 1), ("length", 2), ("width", 1)],
        )],
        &[0],
    );
    assert_eq!(result.intermediates[&0], result.output);
    let Value::ForensicReport {
        data: ForensicReport::Hex { rows },
    } = &result.output
    else {
        panic!()
    };
    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].offset, rows[1].offset), (1, 2));
    assert_eq!(rows[0].bytes, [0]);
    assert_eq!(rows[1].bytes, [255]);
    assert_eq!(rows[1].hex, "ff");
    assert_eq!(rows[1].ascii, ".");
    let wire = serde_json::to_string(&result.output).unwrap();
    assert_eq!(pipeline::json::value(&wire).unwrap(), result.output);
    assert!(result.output.size() <= result.work_bytes_upper_bound as usize);
}
#[test]
fn decoders_compose_with_structured_observations_and_legacy_recipes() {
    let plan = Plan::validate(
        Kind::Text,
        vec![
            Step::new("encoding.hex.decode"),
            Step::new("file.signature.detect"),
        ],
        Limits::default(),
    )
    .unwrap();
    let r = plan
        .execute(
            Value::Text {
                data: "504b0304".into(),
            },
            &[0],
            None,
        )
        .unwrap();
    let Value::ForensicReport {
        data: ForensicReport::Signatures { matches },
    } = r.output
    else {
        panic!()
    };
    assert_eq!(matches[0].format, "zip");
    assert_eq!(matches[0].offset, 0);
    assert_eq!(
        r.intermediates[&0],
        Value::Bytes {
            data: b"PK\x03\x04".to_vec()
        }
    );
    let recipe=pipeline::recipe::Recipe::parse(r#"{"format_version":1,"engine_api":1,"input":{"source":"inline","value":{"type":"bytes","base64":"QUJD"}},"output":{"type":"forensic_report"},"steps":[{"operation":"file.strings.extract","version":1,"parameters":{"min_length":3}}]}"#).unwrap();
    assert_eq!(recipe.info().plan.output_type, Kind::ForensicReport);
}
#[test]
fn printable_runs_keep_completeness_and_absolute_offsets() {
    let r = run(
        b"\0hello\0world\0",
        vec![step("file.strings.extract", &[("max_results", 1)])],
        &[],
    );
    let Value::ForensicReport {
        data:
            ForensicReport::Strings {
                strings,
                complete,
                scanned_bytes,
            },
    } = r.output
    else {
        panic!()
    };
    assert_eq!(strings[0].text, "hello");
    assert_eq!(strings[0].offset, 1);
    assert_eq!(strings[0].length, 5);
    assert!(!complete);
    assert!(scanned_bytes < 13);
    for id in [
        "file.signature.detect",
        "file.hex.inspect",
        "file.strings.extract",
    ] {
        let r = run(b"", vec![Step::new(id)], &[]);
        r.output.validate(pipeline::MAX_VALUE_BYTES).unwrap();
    }
}
#[test]
fn forensic_preflight_rejects_types_parameters_versions_and_io() {
    for params in [
        vec![("width", 0)],
        vec![("offset", -1)],
        vec![("length", -1)],
        vec![("width", 257)],
    ] {
        assert!(
            Plan::validate(
                Kind::Bytes,
                vec![step("file.hex.inspect", &params)],
                Limits::default()
            )
            .is_err()
        );
    }
    for (id, params) in [
        ("file.strings.extract", vec![("min_length", 0)]),
        ("file.strings.extract", vec![("max_results", 10001)]),
    ] {
        assert!(Plan::validate(Kind::Bytes, vec![step(id, &params)], Limits::default()).is_err());
    }
    for steps in [
        vec![Step::new("file.hex.inspect"), Step::new("hash.sha256")],
        vec![Step::new("file.range.extract")],
    ] {
        assert!(Plan::validate(Kind::Bytes, steps, Limits::default()).is_err());
    }
    let mut bad = Step::new("file.hex.inspect");
    bad.version = 2;
    assert!(Plan::validate(Kind::Bytes, vec![bad], Limits::default()).is_err());
    assert!(
        Plan::validate(
            Kind::Text,
            vec![Step::new("file.hex.inspect")],
            Limits::default()
        )
        .is_err()
    );
}
#[test]
fn limits_and_ranges_are_checked_before_algorithms() {
    // Invalid hex would fail at step 0 if conservative bounds did not preflight all steps.
    let plan = Plan::validate(
        Kind::Text,
        vec![
            Step::new("encoding.hex.decode"),
            step("file.hex.inspect", &[("offset", 9)]),
        ],
        Limits::default(),
    )
    .unwrap();
    let e = plan
        .execute(Value::Text { data: "zz".into() }, &[], None)
        .unwrap_err();
    assert_eq!(e.step, Some(1));
    for limits in [
        Limits {
            max_value_bytes: 100,
            ..Limits::default()
        },
        Limits {
            max_work_bytes: 100,
            ..Limits::default()
        },
        Limits {
            max_capture_bytes: 100,
            ..Limits::default()
        },
    ] {
        let p = Plan::validate(Kind::Bytes, vec![Step::new("file.hex.inspect")], limits).unwrap();
        assert!(
            p.execute(Value::Bytes { data: vec![0; 10] }, &[0], None)
                .is_err()
        );
    }
    let p = Plan::validate(
        Kind::Bytes,
        vec![step("file.hex.inspect", &[("width", 1)])],
        Limits::default(),
    )
    .unwrap();
    assert!(
        p.execute(
            Value::Bytes {
                data: vec![0; 10000]
            },
            &[],
            None
        )
        .is_err()
    );
    // The exact conservative hex bound for one byte/row is 64 + 8 + 160.
    let p = Plan::validate(
        Kind::Bytes,
        vec![Step::new("file.hex.inspect")],
        Limits {
            max_value_bytes: 232,
            max_work_bytes: 234,
            max_capture_bytes: 232,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(
        p.execute(Value::Bytes { data: vec![255] }, &[0], None)
            .is_ok()
    );
}
#[test]
fn forged_structured_values_are_bounded_and_consistent() {
    for json in [
        r#"{"type":"forensic_report","data":{"kind":"hex","rows":[{"offset":0,"bytes":"/w==","hex":"00","ascii":"."}]}}"#,
        r#"{"type":"forensic_report","data":{"kind":"strings","strings":[{"offset":18446744073709551615,"length":1,"text":"A"}],"scanned_bytes":1,"complete":true}}"#,
        r#"{"type":"forensic_report","data":{"kind":"signatures","matches":[{"format":"zip","mime":"x","offset":0,"evidence_hex":"!","rule":"test"}]}}"#,
        r#"{"type":"forensic_report","data":{"kind":"arbitrary","data":{}}}"#,
    ] {
        assert!(pipeline::json::value(json).is_err());
    }
    for a in pipeline::supported_adapters() {
        if a.operation.starts_with("file.") {
            assert_eq!(a.kinds().unwrap(), (Kind::Bytes, Kind::ForensicReport));
        }
    }
}
