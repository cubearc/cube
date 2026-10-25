use cube_core::{
    errors::ErrorCode,
    hashing,
    pipeline::{Kind, Limits, Parameter, Plan, Step, Value},
    web::assets::{self, HtmlSelector},
};

fn har(text: Option<&str>, encoding: Option<&str>) -> String {
    let mut content = serde_json::json!({"mimeType":"application/octet-stream","size":-1,"_file":"../never-opened"});
    if let Some(text) = text {
        content["text"] = text.into();
    }
    if let Some(encoding) = encoding {
        content["encoding"] = encoding.into();
    }
    serde_json::json!({"log":{"version":"1.2","entries":[{"request":{"method":"GET","url":"file:///never-opened"},"response":{"status":200,"content":content}}]}}).to_string()
}

#[test]
fn explicit_html_assets_preserve_source_ranges_and_decoded_binary_provenance() {
    let document = "雪<script>\r\n&copy;\0é</script><style> inert </style><img src='data:application/octet-stream;BASE64,AP8&#61;'>";
    let result = assets::html(document, 0, HtmlSelector::Inline).unwrap();
    assert_eq!(result.data, "\r\n&copy;\0é".as_bytes());
    let [start, end] = result.provenance.source_range.unwrap();
    assert_eq!(result.data, document.as_bytes()[start..end]);
    assert_eq!(
        result.provenance.artifact_sha256,
        hashing::sha256(document.as_bytes())
    );
    assert_eq!(result.provenance.sha256, hashing::sha256(&result.data));
    assert_eq!(result.provenance.bytes, result.data.len());
    assert_eq!(result.provenance.representation, "exact_source_utf8");
    assert_eq!(
        assets::html(document, 1, HtmlSelector::Inline)
            .unwrap()
            .data,
        b" inert "
    );
    let result = assets::html(document, 0, HtmlSelector::DataUrl).unwrap();
    assert_eq!(result.data, [0, 255]);
    assert_eq!(
        result.provenance.media_type.as_deref(),
        Some("application/octet-stream")
    );
    assert_eq!(
        result.provenance.representation,
        "html_character_decoding_then_base64"
    );
    assert_eq!(result.into_value(), Value::Bytes { data: vec![0, 255] });
    assert!(
        assets::html("<script></script>", 0, HtmlSelector::Inline)
            .unwrap()
            .data
            .is_empty()
    );
    assert!(
        assets::html("<img src='data:;base64,'>", 0, HtmlSelector::DataUrl)
            .unwrap()
            .data
            .is_empty()
    );
}

#[test]
fn selection_does_not_resolve_references_or_guess_invalid_encodings() {
    for source in [
        "../private",
        "file:///private",
        "https://invalid.example/x",
        "//invalid.example/x",
        "data:,hello",
        "data:;base64",
        "data:;base64,AB==",
        "data:;base64,%41P8=",
        "data:;base64,AP8",
        "data:;base64, AP8=",
        "data:;base64,_w==",
    ] {
        let document = format!("<img src='{source}'>");
        let error = assets::html(&document, 0, HtmlSelector::DataUrl).unwrap_err();
        assert_eq!(error.operation, Some("web.html.asset"));
    }
    for document in ["", "<script>unterminated"] {
        assert!(assets::html(document, 0, HtmlSelector::Inline).is_err());
    }
    assert!(assets::html("<script>x</script>", usize::MAX, HtmlSelector::Inline).is_err());
    assert!(HtmlSelector::parse("automatic").is_err());
    let oversized = format!(
        "<script>{}</script>",
        "x".repeat(assets::MAX_ASSET_BYTES + 1)
    );
    assert_eq!(
        assets::html(&oversized, 0, HtmlSelector::Inline)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let exact = format!("<script>{}</script>", "x".repeat(assets::MAX_ASSET_BYTES));
    assert_eq!(
        assets::html(&exact, 0, HtmlSelector::Inline)
            .unwrap()
            .data
            .len(),
        assets::MAX_ASSET_BYTES
    );
}

#[test]
fn har_body_representation_is_explicit_and_unknown_or_external_bodies_fail() {
    let source = har(Some("AP8="), Some("base64"));
    let result = assets::har(&source, 0).unwrap();
    assert_eq!(result.data, [0, 255]);
    assert_eq!(
        result.provenance.source_pointer,
        "/log/entries/0/response/content/text"
    );
    assert_eq!(result.provenance.source_range, None);
    assert_eq!(result.provenance.representation, "har_base64_decoded_bytes");
    assert_eq!(
        result.provenance.artifact_sha256,
        hashing::sha256(source.as_bytes())
    );
    assert_eq!(result.provenance.sha256, hashing::sha256(&[0, 255]));
    let result = assets::har(&har(Some("é\0\r\n"), None), 0).unwrap();
    assert_eq!(result.data, "é\0\r\n".as_bytes());
    assert_eq!(result.provenance.representation, "har_logical_text_utf8");
    assert!(
        assets::har(&har(Some(""), None), 0)
            .unwrap()
            .data
            .is_empty()
    );
    for source in [
        har(None, None),
        har(Some("AP8="), Some("gzip")),
        har(Some("AB=="), Some("base64")),
        "{".into(),
    ] {
        assert_eq!(
            assets::har(&source, 0).unwrap_err().operation,
            Some("web.har.asset")
        );
    }
    assert!(assets::har(&source, usize::MAX).is_err());
    let exact = har(Some(&"x".repeat(assets::MAX_ASSET_BYTES)), None);
    assert_eq!(
        assets::har(&exact, 0).unwrap().data.len(),
        assets::MAX_ASSET_BYTES
    );
    assert_eq!(
        assets::har(
            &har(Some(&"x".repeat(assets::MAX_ASSET_BYTES + 1)), None),
            0
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
}

fn step(operation: &str) -> Step {
    let mut step = Step::new(operation);
    step.parameters
        .insert("index".into(), Parameter::Integer(0));
    step
}

#[test]
fn asset_adapters_reuse_existing_engine_types_captures_preflight_and_budgets() {
    for (operation, document) in [
        ("web.html.asset", "<script>00ff</script>".into()),
        ("web.har.asset", har(Some("00ff"), None)),
    ] {
        let steps = vec![
            step(operation),
            Step::new("text.utf8.decode"),
            Step::new("encoding.hex.decode"),
            Step::new("file.hex.inspect"),
        ];
        let plan = Plan::validate(Kind::Text, steps, Limits::default()).unwrap();
        let result = plan
            .execute(Value::Text { data: document }, &[0, 2], None)
            .unwrap();
        assert_eq!(
            result.intermediates[&0],
            Value::Bytes {
                data: b"00ff".to_vec()
            }
        );
        assert_eq!(
            result.intermediates[&2],
            Value::Bytes { data: vec![0, 255] }
        );
        assert_eq!(result.output.kind(), Kind::ForensicReport);
        assert!(Plan::validate(Kind::Bytes, vec![step(operation)], Limits::default()).is_err());
        let error = Plan::validate(
            Kind::Text,
            vec![step(operation), Step::new("cipher.rot13")],
            Limits::default(),
        )
        .unwrap_err();
        assert_eq!(error.step, Some(1));
        assert!(Plan::validate(Kind::Text, vec![Step::new(operation)], Limits::default()).is_err());
        let mut version = step(operation);
        version.version = 2;
        assert!(Plan::validate(Kind::Text, vec![version], Limits::default()).is_err());
        let mut invalid = step(operation);
        invalid
            .parameters
            .insert("index".into(), Parameter::Integer(-1));
        assert!(Plan::validate(Kind::Text, vec![invalid], Limits::default()).is_err());
        let mut invalid = step(operation);
        invalid
            .parameters
            .insert("selector".into(), Parameter::Text("guess".into()));
        assert!(Plan::validate(Kind::Text, vec![invalid], Limits::default()).is_err());
    }
    let mut selected = step("web.html.asset");
    selected
        .parameters
        .insert("selector".into(), Parameter::Text("data_url".into()));
    let result = Plan::validate(Kind::Text, vec![selected.clone()], Limits::default())
        .unwrap()
        .execute(
            Value::Text {
                data: "<img src='data:;base64,AP8='>".into(),
            },
            &[],
            None,
        )
        .unwrap();
    assert_eq!(result.output, Value::Bytes { data: vec![0, 255] });
    let limited = Plan::validate(
        Kind::Text,
        vec![selected],
        Limits {
            max_capture_bytes: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        limited
            .execute(
                Value::Text {
                    data: "<img src='data:;base64,AP8='>".into()
                },
                &[0],
                None
            )
            .unwrap_err()
            .error
            .code,
        ErrorCode::ResourceLimit
    );
    let plan = Plan::validate(Kind::Text, vec![step("web.har.asset")], Limits::default()).unwrap();
    let error = plan
        .execute(
            Value::Text {
                data: "x".repeat(256 * 1024 + 1),
            },
            &[],
            None,
        )
        .unwrap_err();
    assert_eq!(error.error.code, ErrorCode::ResourceLimit);
    assert_eq!(error.operation.as_deref(), Some("web.har.asset"));
}
