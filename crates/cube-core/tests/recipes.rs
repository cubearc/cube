use cube_core::pipeline::{self, Value, recipe::Recipe};
use serde_json::{Value as Json, json};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
fn recipe() -> Json {
    json!({"format_version":1,"engine_api":1,"input":{"source":"slot","name":"payload","type":"bytes"},"output":{"type":"bytes"},"steps":[{"operation":"encoding.hex.encode","version":1},{"operation":"encoding.hex.decode","version":1}]})
}
fn bytes(data: &[u8]) -> Value {
    Value::Bytes {
        data: data.to_vec(),
    }
}
fn bindings() -> BTreeMap<String, Value> {
    BTreeMap::from([("payload".into(), bytes(&[0, 255]))])
}
#[test]
fn recipes_execute_exact_typed_contracts_deterministically_without_capabilities() {
    let recipe = Recipe::parse(&recipe().to_string()).unwrap();
    let info = serde_json::to_value(recipe.info()).unwrap();
    assert_eq!(info["format_version"], 1);
    assert_eq!(info["input_slot"], "payload");
    assert_eq!(info["plan"]["engine_api"], 1);
    for _ in 0..2 {
        let result = recipe.execute(&bindings(), &[0], None).unwrap();
        assert_eq!(result.output, bytes(&[0, 255]));
        assert_eq!(
            result.intermediates[&0],
            Value::Text {
                data: "00ff".into()
            }
        );
    }
    assert!(recipe.check(&bindings(), &bytes(&[0, 255])).unwrap().passed);
    assert!(recipe.check(&bindings(), &bytes(&[255, 0])).is_err());
    assert!(
        recipe
            .check(
                &bindings(),
                &Value::Text {
                    data: "00ff".into()
                }
            )
            .is_err()
    );
    assert!(
        recipe
            .execute(&bindings(), &[], Some(&AtomicBool::new(true)))
            .is_err()
    );
    for bindings in [
        BTreeMap::new(),
        BTreeMap::from([("other".into(), bytes(&[]))]),
        BTreeMap::from([("payload".into(), bytes(&[])), ("extra".into(), bytes(&[]))]),
        BTreeMap::from([("payload".into(), Value::Text { data: "".into() })]),
    ] {
        assert!(recipe.execute(&bindings, &[], None).is_err());
    }
    let oversized = BTreeMap::from([(
        "payload".into(),
        bytes(&vec![0; pipeline::MAX_VALUE_BYTES + 1]),
    )]);
    assert!(recipe.execute(&oversized, &[], None).is_err());
}
#[test]
fn inline_values_are_typed_redacted_and_never_resolve_paths_or_environment() {
    for value in [
        json!({"type":"bytes","base64":"AP8="}),
        json!({"type":"text","data":"$HOME/secret\n\u{1f9ca}"}),
        json!({"type":"byte_range","offset":2,"length":3}),
    ] {
        let mut definition = recipe();
        definition["input"] = json!({"source":"inline","value":value});
        definition["output"]["type"] = value["type"].clone();
        definition["steps"] = json!([]);
        let recipe = Recipe::parse(&definition.to_string()).unwrap();
        let expected = pipeline::json::value(&value.to_string()).unwrap();
        assert_eq!(
            recipe.execute(&BTreeMap::new(), &[], None).unwrap().output,
            expected
        );
        assert!(recipe.execute(&bindings(), &[], None).is_err());
        assert!(recipe.check(&BTreeMap::new(), &expected).unwrap().passed);
        let info = serde_json::to_string(&recipe.info()).unwrap();
        assert!(!info.contains("$HOME"));
        assert!(!info.contains("AP8="));
        assert!(recipe.info().input_slot.is_none());
    }
}
#[test]
fn compatibility_and_complete_schema_validation_precede_execution() {
    let original = recipe();
    for (field, value) in [
        ("format_version", json!(2)),
        ("engine_api", json!(2)),
        ("format_version", json!(1.0)),
        ("shell", json!("echo unsafe")),
        ("includes", json!(["secret"])),
    ] {
        let mut invalid = original.clone();
        invalid[field] = value;
        assert!(Recipe::parse(&invalid.to_string()).is_err(), "{field}");
    }
    for invalid in [
        json!({"source":"file","path":"secret"}),
        json!({"source":"slot","name":"../secret","type":"bytes"}),
        json!({"source":"slot","name":"payload","type":"bytes","path":"secret"}),
        json!({"source":"inline","value":{"type":"bytes","base64":"AB=="}}),
        json!({"source":"inline","value":{"type":"bytes","base64":"AP8="},"name":"payload"}),
    ] {
        let mut document = original.clone();
        document["input"] = invalid;
        assert!(Recipe::parse(&document.to_string()).is_err());
    }
    for invalid in [
        json!({"operation":"encoding.hex.encode"}),
        json!({"operation":"encoding.hex.encode","version":2}),
        json!({"operation":"shell.exec","version":1}),
        json!({"operation":"encoding.hex.encode","version":1,"parameters":{"unexpected":true}}),
        json!({"operation":"encoding.hex.encode","version":1,"extra":true}),
    ] {
        let mut document = original.clone();
        document["steps"][0] = invalid;
        assert_eq!(
            Recipe::parse(&document.to_string()).unwrap_err().step,
            Some(0)
        );
    }
    let mut invalid = original.clone();
    invalid["output"]["type"] = json!("text");
    assert!(Recipe::parse(&invalid.to_string()).is_err());
    invalid = original.clone();
    invalid["steps"] = json!(vec![json!({"operation":"cipher.rot13","version":1}); 65]);
    assert!(Recipe::parse(&invalid.to_string()).is_err());
    invalid = original.clone();
    invalid["limits"] = json!({"max_steps":1});
    assert!(Recipe::parse(&invalid.to_string()).is_err());
    invalid = original.clone();
    invalid["limits"] = json!({"max_value_bytes":1});
    let low = Recipe::parse(&invalid.to_string()).unwrap();
    assert!(low.execute(&bindings(), &[], None).is_err());
    invalid = original.clone();
    invalid["input"] = json!({"source":"inline","value":{"type":"bytes","base64":"AP8="}});
    invalid["limits"] = json!({"max_value_bytes":1});
    assert!(Recipe::parse(&invalid.to_string()).is_err());
    assert!(Recipe::parse(r#"{"format_version":1,"format_version":1}"#).is_err());
    assert!(Recipe::parse(&" ".repeat(pipeline::json::MAX_DOCUMENT_BYTES + 1)).is_err());
    for name in ["", "a/b", "$HOME", &"a".repeat(65)] {
        assert!(pipeline::recipe::validate_slot(name).is_err());
    }
    for name in ["payload", "a-B_12", &"a".repeat(64)] {
        pipeline::recipe::validate_slot(name).unwrap();
    }
}

#[test]
fn validation_does_not_run_even_a_runtime_invalid_inline_operation() {
    let definition = r#"{"format_version":1,"engine_api":1,"input":{"source":"inline","value":{"type":"text","data":"zz"}},"output":{"type":"bytes"},"steps":[{"operation":"encoding.hex.decode","version":1}]}"#;
    let recipe = Recipe::parse(definition).unwrap();
    let error = recipe.execute(&BTreeMap::new(), &[], None).unwrap_err();
    assert_eq!(error.step, Some(0));
    assert_eq!(error.operation.as_deref(), Some("encoding.hex.decode"));
}
