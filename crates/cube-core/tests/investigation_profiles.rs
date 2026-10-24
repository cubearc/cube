use cube_core::{
    investigation::{
        profile::{Budgets, Depth, Pipeline, Profile},
        session::{Export, Session, State},
    },
    pipeline::{Limits, Step},
};
use std::{
    io::{self, Cursor, Read, Seek, SeekFrom},
    sync::atomic::AtomicBool,
};
struct Counted {
    inner: Cursor<Vec<u8>>,
    read: usize,
}
impl Read for Counted {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(out)?;
        self.read += n;
        Ok(n)
    }
}
impl Seek for Counted {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}
fn reader(data: &[u8]) -> Counted {
    Counted {
        inner: Cursor::new(data.to_vec()),
        read: 0,
    }
}
fn preview() -> Profile {
    Profile {
        depth: Depth::Preview,
        ..Default::default()
    }
}
#[test]
fn dry_run_and_execution_expose_bounded_ranges_and_deterministic_provenance() {
    let data = b"\0original printable text\0\xff";
    let p = Profile {
        identify: true,
        ..preview()
    };
    let plan = p.dry_run(data.len() as u64).unwrap();
    assert_eq!(
        plan.runs.iter().map(|s| s.operation).collect::<Vec<_>>(),
        vec![
            "hash.sha256",
            "investigation.triage",
            "file.hex.inspect",
            "file.strings.extract"
        ]
    );
    assert_eq!(plan.input_read_bytes, data.len());
    assert_eq!(plan.operation_units, 4);
    let mut source = reader(data);
    let result = p.run_reader(&mut source, None).unwrap();
    assert_eq!(source.read, data.len());
    assert_eq!(result.input_read_bytes, data.len());
    assert_eq!(result.operation_units, 4);
    assert!(!result.session.partial());
    assert_eq!(result.session.runs().len(), 4);
    assert!(result.session.runs().iter().all(|r| r.retained.is_none()));
    let record = result.session.to_json(&Export::default()).unwrap();
    assert!(!record.contains("original printable text"));
    assert!(record.contains("triage.unknown_strings.v1"));
    let second = p.run_bytes(data, None).unwrap();
    assert_eq!(record, second.session.to_json(&Export::default()).unwrap());
    assert!(
        Session::from_json(&record)
            .unwrap()
            .replay(data, None)
            .is_ok()
    );
    let mut big = reader(&vec![0; 1024 * 1024 + 1]);
    let result = Profile::default().run_reader(&mut big, None).unwrap();
    assert_eq!(big.read, 12);
    assert_eq!(result.input_read_bytes, 12);
    assert!(!result.session.partial());
    let mut source = reader(b"\x89PNG\r\n\x1a\n");
    let result = Profile::default().run_reader(&mut source, None).unwrap();
    assert!(result.session.human().contains("triage.png_chunks.v1"));
    assert_eq!(source.read, 8);
}
#[test]
fn exact_and_exceeded_budgets_never_silently_expand_work_or_retention() {
    let data = b"abcdef";
    let exact = Profile {
        budgets: Budgets {
            max_input_bytes: 6,
            max_inspected_bytes: 18,
            max_operations: 3,
            ..Default::default()
        },
        ..preview()
    };
    let result = exact.run_bytes(data, None).unwrap();
    assert!(!result.session.partial());
    assert_eq!(result.inspected_bytes, 18);
    for budgets in [
        Budgets {
            max_input_bytes: 5,
            ..exact.budgets.clone()
        },
        Budgets {
            max_inspected_bytes: 17,
            ..exact.budgets.clone()
        },
        Budgets {
            max_operations: 2,
            ..exact.budgets.clone()
        },
    ] {
        let p = Profile {
            budgets,
            ..preview()
        };
        let mut source = reader(data);
        let result = p.run_reader(&mut source, None).unwrap();
        assert!(result.session.partial());
        assert!(
            result
                .session
                .runs()
                .iter()
                .any(|r| r.state == State::Skipped)
        );
        assert!(source.read <= p.budgets.max_input_bytes);
        assert!(result.inspected_bytes <= p.budgets.max_inspected_bytes);
        assert!(result.operation_units <= p.budgets.max_operations);
        let record = result.session.to_json(&Export::default()).unwrap();
        assert!(Session::from_json(&record).is_ok());
    }
    let retain = Profile {
        retain_results: vec![2],
        budgets: Budgets {
            max_retained_bytes: 1144,
            ..Default::default()
        },
        ..preview()
    };
    let result = retain.run_bytes(data, None).unwrap();
    assert!(!result.session.partial());
    assert!(result.session.runs()[1].retained.is_some());
    let selected = result
        .session
        .to_json(&Export {
            include_results: vec![2],
            ..Export::default()
        })
        .unwrap();
    assert!(selected.contains("616263646566"));
    assert!(Session::from_json(&selected).is_ok());
    let low = Profile {
        budgets: Budgets {
            max_retained_bytes: 1143,
            ..Default::default()
        },
        ..retain
    };
    let result = low.run_bytes(data, None).unwrap();
    assert!(result.session.partial());
    assert_eq!(result.session.runs()[1].state, State::Skipped);
    assert!(
        result
            .session
            .to_json(&Export {
                include_results: vec![2],
                ..Export::default()
            })
            .is_ok()
    );
    let zero = Profile {
        budgets: Budgets {
            max_operations: 0,
            max_input_bytes: 0,
            max_inspected_bytes: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut source = reader(data);
    let result = zero.run_reader(&mut source, None).unwrap();
    assert_eq!(source.read, 0);
    assert!(result.session.partial());
}
#[test]
fn cancellation_deadline_and_invalid_later_steps_prevent_unexpected_reads() {
    let mut source = reader(b"data");
    let result = preview()
        .run_reader(&mut source, Some(&AtomicBool::new(true)))
        .unwrap();
    assert_eq!(source.read, 0);
    assert_eq!(result.operation_units, 0);
    assert!(
        result
            .session
            .runs()
            .iter()
            .all(|r| r.state == State::Skipped)
    );
    assert_eq!(
        result.session.runs()[0].diagnostic.as_ref().unwrap().code,
        "cancelled"
    );
    let zero = Profile {
        budgets: Budgets {
            max_elapsed_ms: 0,
            ..Default::default()
        },
        ..preview()
    };
    let mut source = reader(b"data");
    let result = zero.run_reader(&mut source, None).unwrap();
    assert_eq!(source.read, 0);
    assert!(result.session.partial());
    assert!(result.session.human().contains("elapsed-time budget"));
    let invalid = Profile {
        pipeline: Some(Pipeline {
            steps: vec![Step::new("encoding.hex.encode"), Step::new("shell.exec")],
            limits: Limits::default(),
        }),
        ..preview()
    };
    let mut source = reader(b"data");
    assert!(invalid.run_reader(&mut source, None).is_err());
    assert_eq!(source.read, 0);
    for document in [
        r#"{"format_version":2}"#,
        r#"{"format_version":1,"shell":"echo secret"}"#,
        r#"{"format_version":1,"budgets":{"max_input_bytes":1048577}}"#,
        r#"{"format_version":1,"depth":"deep"}"#,
        r#"{"format_version":1,"retain_results":[1,1]}"#,
        r#"{"format_version":1,"format_version":1}"#,
    ] {
        assert!(Profile::parse(document).is_err(), "{document}");
    }
    assert!(
        Profile {
            retain_results: vec![99],
            ..Default::default()
        }
        .dry_run(4)
        .is_err()
    );
}
#[test]
fn profiles_reuse_pipeline_contracts_and_record_failed_or_truncated_outcomes() {
    let p = Profile {
        identify: true,
        pipeline: Some(Pipeline {
            steps: vec![
                Step::new("encoding.hex.encode"),
                Step::new("encoding.hex.decode"),
            ],
            limits: Limits::default(),
        }),
        ..Default::default()
    };
    let result = p.run_bytes(&[0, 255], None).unwrap();
    assert!(!result.session.partial());
    assert_eq!(result.operation_units, 5);
    assert_eq!(
        result.session.runs().last().unwrap().operation,
        "pipeline.execute"
    );
    assert!(result.session.replay(&[0, 255], None).is_ok());
    let failure = Profile {
        pipeline: Some(Pipeline {
            steps: vec![
                Step::new("text.utf8.decode"),
                Step::new("encoding.hex.decode"),
            ],
            limits: Limits::default(),
        }),
        ..Default::default()
    };
    let result = failure.run_bytes(b"zz", None).unwrap();
    assert!(result.session.partial());
    assert_eq!(result.session.runs()[1].state, State::Failed);
    let mut data = Vec::new();
    for _ in 0..40 {
        data.extend_from_slice(b"text\0");
    }
    let result = preview().run_bytes(&data, None).unwrap();
    assert!(result.session.partial());
    assert!(result.session.human().contains("incomplete"));
    let pipeline = p.pipeline.unwrap();
    let selected = Profile {
        pipeline: Some(pipeline),
        retain_results: vec![2],
        ..Default::default()
    };
    assert!(!selected.dry_run(2).unwrap().runs[1].execute);
}

#[test]
fn optional_parameter_recording_is_preflighted_before_any_source_reads() {
    use cube_core::pipeline::Parameter;
    use std::collections::BTreeMap;
    let step = Step {
        operation: "binary.xor".into(),
        version: 1,
        parameters: BTreeMap::from([(
            "key".into(),
            Parameter::Bytes {
                data: vec![42; 65536],
            },
        )]),
    };
    let profile = Profile {
        record_parameters: true,
        pipeline: Some(Pipeline {
            steps: vec![step.clone(), step],
            limits: Limits::default(),
        }),
        ..Default::default()
    };
    let mut source = reader(b"data");
    assert!(profile.run_reader(&mut source, None).is_err());
    assert_eq!(source.read, 0);
}
