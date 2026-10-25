use cube_core::{
    investigation::session::{Capture, Export, RunSpec, Session, SessionLimits, State},
    pipeline::{Limits, Parameter, Step},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
fn spec() -> RunSpec {
    RunSpec::pipeline(
        vec![
            Step::new("encoding.hex.encode"),
            Step::new("encoding.hex.decode"),
        ],
        Limits::default(),
    )
}
fn identified(data: &[u8]) -> Session {
    let mut s = Session::new(data.len() as u64, SessionLimits::default()).unwrap();
    s.execute(data, RunSpec::Identity, Capture::default(), None)
        .unwrap();
    s
}
#[test]
fn provenance_is_typed_private_by_default_and_replay_verifies_all_contracts() {
    let data = b"\x00\xffPRIVATE\n";
    let mut session = identified(data);
    let run = session
        .execute(
            data,
            spec(),
            Capture {
                parameters: false,
                result: true,
            },
            None,
        )
        .unwrap();
    assert_eq!(run, 2);
    assert_eq!(session.runs()[1].state, State::Succeeded);
    assert!(!session.findings().is_empty());
    assert!(session.replay_gaps().is_empty());
    session
        .add_hypothesis("Maybe \x1b[31m\nsecret\u{202e}", &[2])
        .unwrap();
    assert!(!session.human().contains('\x1b'));
    assert!(!session.human().contains('\u{202e}'));
    assert!(session.human().contains("User hypothesis"));
    let record = session.to_json(&Export::default()).unwrap();
    assert!(!record.contains("PRIVATE"));
    assert!(!record.contains("Maybe"));
    let document: Value = serde_json::from_str(&record).unwrap();
    assert_eq!(document["status"], "complete");
    assert!(document["session"]["runs"][1]["retained"].is_null());
    let loaded = Session::from_json(&record).unwrap();
    let replay = loaded.replay(data, None).unwrap();
    assert_eq!(replay.runs().len(), 2);
    assert_eq!(
        replay.runs()[1].result_digest,
        loaded.runs()[1].result_digest
    );
    assert!(loaded.replay(b"\x01\xffPRIVATE\n", None).is_err());
    assert!(loaded.replay(b"other", None).is_err());
    let selected = session
        .to_json(&Export {
            include_results: vec![2],
            include_parameters: vec![],
            include_hypotheses: true,
        })
        .unwrap();
    assert!(selected.contains("Maybe"));
    let loaded = Session::from_json(&selected).unwrap();
    assert!(loaded.runs()[1].retained.is_some());
    session.discard_result(2).unwrap();
    assert!(
        session
            .to_json(&Export {
                include_results: vec![2],
                ..Export::default()
            })
            .is_err()
    );
    assert!(session.discard_result(99).is_err());
    assert!(
        session
            .to_json(&Export {
                include_results: vec![99],
                ..Export::default()
            })
            .is_err()
    );
    assert!(
        session
            .to_json(&Export {
                include_parameters: vec![2, 2],
                ..Export::default()
            })
            .is_err()
    );
    assert!(session.add_hypothesis("bad reference", &[999]).is_err());
    assert!(
        session
            .add_hypothesis("duplicate reference", &[2, 2])
            .is_err()
    );
}
#[test]
fn keyed_parameters_are_redacted_and_cannot_silently_restore_replay() {
    let data = b"hello";
    let step = Step {
        operation: "binary.xor".into(),
        version: 1,
        parameters: BTreeMap::from([(
            "key".into(),
            Parameter::Bytes {
                data: b"secret-key".to_vec(),
            },
        )]),
    };
    let mut session = identified(data);
    session
        .execute(
            data,
            RunSpec::pipeline(vec![step.clone()], Limits::default()),
            Capture::default(),
            None,
        )
        .unwrap();
    let default = session.to_json(&Export::default()).unwrap();
    assert!(!default.contains("c2VjcmV0LWtleQ=="));
    assert!(default.contains("redacted"));
    assert!(
        Session::from_json(&default)
            .unwrap()
            .replay(data, None)
            .is_err()
    );
    assert!(
        session
            .to_json(&Export {
                include_parameters: vec![2],
                ..Export::default()
            })
            .is_err()
    );
    let mut explicit = identified(data);
    explicit
        .execute(
            data,
            RunSpec::pipeline(vec![step], Limits::default()),
            Capture {
                parameters: true,
                result: false,
            },
            None,
        )
        .unwrap();
    let record = explicit
        .to_json(&Export {
            include_parameters: vec![2],
            ..Export::default()
        })
        .unwrap();
    assert!(record.contains("c2VjcmV0LWtleQ=="));
    assert!(
        Session::from_json(&record)
            .unwrap()
            .replay(data, None)
            .is_ok()
    );
    explicit.redact_parameters(2).unwrap();
    assert!(!explicit.replay_gaps().is_empty());
    assert!(explicit.redact_parameters(99).is_err());
    let no_identity = Session::new(5, SessionLimits::default()).unwrap();
    assert!(!no_identity.replay_gaps().is_empty());
    assert!(no_identity.replay(data, None).is_err());
}
#[test]
fn failed_cancelled_and_bounded_runs_preserve_state_without_raw_contents() {
    let data = b"zz";
    let mut s = identified(data);
    let failed = RunSpec::pipeline(
        vec![
            Step::new("text.utf8.decode"),
            Step::new("encoding.hex.decode"),
        ],
        Limits::default(),
    );
    s.execute(data, failed, Capture::default(), None).unwrap();
    assert_eq!(s.runs()[1].state, State::Failed);
    assert!(
        s.runs()[1]
            .diagnostic
            .as_ref()
            .unwrap()
            .message
            .contains("step 1")
    );
    assert!(s.partial());
    let loaded = Session::from_json(&s.to_json(&Export::default()).unwrap()).unwrap();
    assert!(loaded.replay(data, None).is_ok());
    s.execute(
        data,
        spec(),
        Capture::default(),
        Some(&AtomicBool::new(true)),
    )
    .unwrap();
    assert_eq!(s.runs()[2].diagnostic.as_ref().unwrap().code, "cancelled");
    let mut limited = Session::new(
        2,
        SessionLimits {
            max_retained_bytes: 0,
            max_findings: 0,
            ..Default::default()
        },
    )
    .unwrap();
    limited
        .execute(
            data,
            spec(),
            Capture {
                parameters: false,
                result: true,
            },
            None,
        )
        .unwrap();
    assert!(limited.partial());
    assert!(limited.runs()[0].retained.is_none());
    assert!(limited.findings().is_empty());
    let mut limited = Session::new(
        2,
        SessionLimits {
            max_runs: 0,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        limited
            .execute(data, spec(), Capture::default(), None)
            .is_err()
    );
    assert!(limited.runs().is_empty());
    assert!(
        Session::new(
            0,
            SessionLimits {
                max_record_bytes: 0,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        Session::new(
            0,
            SessionLimits {
                max_runs: 65,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(s.add_hypothesis(&"x".repeat(1025), &[]).is_err());
    let mut s = Session::new(
        2,
        SessionLimits {
            max_hypotheses: 0,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(s.add_hypothesis("no", &[]).is_err());
    assert!(
        s.execute(
            data,
            RunSpec::Hex {
                offset: 2,
                length: 1,
                width: 16
            },
            Capture::default(),
            None
        )
        .is_err()
    );
    assert!(
        s.execute(
            data,
            RunSpec::Strings {
                offset: 0,
                length: 2,
                min_length: 0,
                max_results: 1
            },
            Capture::default(),
            None
        )
        .is_err()
    );
}
#[test]
fn record_loading_rejects_duplicate_ids_versions_forged_results_and_references() {
    let mut s = identified(b"abc");
    s.execute(
        b"abc",
        spec(),
        Capture {
            parameters: false,
            result: true,
        },
        None,
    )
    .unwrap();
    let record = s
        .to_json(&Export {
            include_results: vec![2],
            ..Export::default()
        })
        .unwrap();
    let original: Value = serde_json::from_str(&record).unwrap();
    for (pointer, value) in [
        ("/session/runs/1/id", json!(1)),
        ("/session/runs/1/version", json!(2)),
        ("/session/runs/1/operation", json!("shell.exec")),
        ("/session/findings/0/run_id", json!(999)),
        ("/session/input/id", json!(2)),
        ("/session/input/digest/algorithm", json!("unknown")),
        ("/session/runs/1/retained/base64", json!("Zm9yZ2Vk")),
        ("/status", json!("partial")),
        ("/schema_version", json!(2)),
        ("/session/engine_api", json!(2)),
    ] {
        let mut invalid = original.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(
            Session::from_json(&invalid.to_string()).is_err(),
            "{pointer}"
        );
        if pointer.starts_with("/session/") {
            assert!(
                serde_json::from_value::<Session>(invalid["session"].clone()).is_err(),
                "{pointer}"
            );
        }
    }
    assert!(Session::from_json(r#"{"schema_version":1,"schema_version":1}"#).is_err());
    let mut invalid = original.clone();
    invalid["session"]["unexpected"] = json!(true);
    assert!(Session::from_json(&invalid.to_string()).is_err());
    let mut invalid = original.clone();
    invalid["session"]["runs"][1]["parameters"]["steps"][0]
        .as_object_mut()
        .unwrap()
        .remove("version");
    assert!(Session::from_json(&invalid.to_string()).is_err());
}

#[test]
fn sessions_reject_masked_audits_before_execution_instead_of_dropping_evidence() {
    let data = b"ABC";
    let mut session = Session::new(3, SessionLimits::default()).unwrap();
    let steps = cube_core::pipeline::json::steps(r#"[
        {"operation":"text.utf8.decode","version":1},
        {"operation":"text.transform.masked","version":1,"parameters":{"selector":"fibonacci","selected":"atbash"}}
    ]"#).unwrap();
    let error = session
        .execute(
            data,
            RunSpec::pipeline(steps, Limits::default()),
            Capture::default(),
            None,
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("audits require direct pipeline or recipe")
    );
    assert!(session.runs().is_empty());
}
