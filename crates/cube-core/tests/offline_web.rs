use cube_core::{
    errors::ErrorCode,
    web::{har, html},
};
#[test]
fn html_tokens_references_and_inline_ranges_are_observations_without_execution() {
    let document = "雪<body onload=\"run()\"><a href='../private?x=1&amp;y=2'>Text &amp; more</a><script src='https://invalid.example/x.js'>fetch('file:///secret');<img src='not-an-element'></script><style>a{background:url(https://invalid.example)}</style><img src='data:application/octet-stream;base64,AP8='>";
    let r = html::inspect(document).unwrap();
    assert_eq!(r.references.len(), 3);
    assert_eq!(r.references[0].value, "../private?x=1&y=2");
    assert_eq!(
        r.references[0].observations,
        ["contains_parent_path_segment"]
    );
    assert_eq!(r.references[1].kind, "remote");
    assert_eq!(r.references[2].kind, "data");
    assert_eq!(r.event_handler_attributes, 1);
    assert!(r.text.contains("Text & more"));
    assert!(!r.text.contains("fetch"));
    assert_eq!(r.embedded.len(), 2);
    let range = r.embedded[0].source_range;
    assert_eq!(
        &document[range[0]..range[1]],
        "fetch('file:///secret');<img src='not-an-element'>"
    );
    assert!(r.embedded[0].terminated);
    let r = html::inspect("<img src='first' src='second'>").unwrap();
    assert_eq!(r.references[0].value, "first");
    assert!(r.parse_errors > 0);
    assert!(!html::inspect("<script>unfinished").unwrap().embedded[0].terminated);
    assert!(
        cube_core::operations::require("web.html.inspect")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
#[test]
fn html_depth_fields_record_and_input_budgets_fail_safely() {
    for input in [
        "<div>".repeat(129),
        format!("<script>{}</script>", "x".repeat(65537)),
        format!("<img src='{}'>", "x".repeat(65537)),
        "<img src=x>".repeat(257),
        "x".repeat(1024 * 1024 + 1),
    ] {
        assert_eq!(
            html::inspect(&input).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    for input in [
        "",
        "<",
        "<!--x",
        "<a href='javascript:run()'>",
        "<textarea>&lt;img src='x'&gt;</textarea>",
    ] {
        let _ = html::inspect(input).unwrap();
    }
    assert_eq!(cube_core::web::reference_kind("#x"), "fragment");
    assert_eq!(
        cube_core::web::reference_kind("file:///tmp/x"),
        "other_scheme"
    );
    assert_eq!(
        cube_core::web::reference_kind("//invalid.example/x"),
        "remote"
    );
}
fn har_document() -> serde_json::Value {
    serde_json::json!({"log":{"version":"1.2","entries":[{"request":{"method":"GET","url":"../asset\n","headers":[{"name":"Cookie","value":"not-exported"}]},"response":{"status":200,"content":{"mimeType":"application/octet-stream","text":"AP8=","encoding":"base64","size":2}},"time":1.25}]}})
}
#[test]
fn har_uses_bounded_existing_json_policy_and_qualified_observations() {
    let document = har_document().to_string();
    let r = har::inspect(&document).unwrap();
    assert_eq!(r.entries[0].status, 200);
    assert_eq!(r.entries[0].body_encoding.as_deref(), Some("base64"));
    assert_eq!(r.entries[0].body_text_bytes, Some(4));
    assert_eq!(r.entries[0].time_ms, Some(1.25));
    assert!(r.entries[0].started_date_time.is_none());
    assert_eq!(r.entries[0].pointer, "/log/entries/0");
    assert_eq!(
        r.entries[0].observations,
        [
            "contains_control_characters",
            "contains_parent_path_segment"
        ]
    );
    let serialized = serde_json::to_string(&r).unwrap();
    assert!(!serialized.contains("not-exported"));
    assert!(!serialized.contains("AP8="));
    assert!(har::inspect(&("\u{feff}".to_string() + &document)).is_ok());
    assert!(har::inspect(r#"{"log":{"version":"1.2","version":"1.2","entries":[]}}"#).is_err());
    assert!(har::inspect(&"[".repeat(20)).is_err());
    let mut bad = har_document();
    bad["log"]["version"] = "2".into();
    assert_eq!(
        har::inspect(&bad.to_string()).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let mut bad = har_document();
    bad["log"]["entries"] = serde_json::json!(vec![serde_json::json!({}); 129]);
    assert_eq!(
        har::inspect(&bad.to_string()).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut bad = har_document();
    bad["log"]["entries"][0]["request"]["url"] = "x".repeat(8193).into();
    assert!(har::inspect(&bad.to_string()).is_err());
    assert!(
        cube_core::operations::require("web.har.inspect")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
#[test]
fn har_missing_invalid_and_oversized_fields_have_safe_errors() {
    for value in [
        serde_json::json!({}),
        serde_json::json!({"log":{"version":"1.2","entries":[{}]}}),
        serde_json::json!({"log":{"version":"1.2","entries":null}}),
    ] {
        assert!(har::inspect(&value.to_string()).is_err());
    }
    for field in ["method", "url"] {
        let mut v = har_document();
        v["log"]["entries"][0]["request"][field] = serde_json::Value::Null;
        assert!(har::inspect(&v.to_string()).is_err());
    }
    for field in ["status", "content"] {
        let mut v = har_document();
        v["log"]["entries"][0]["response"][field] = (-999).into();
        assert!(har::inspect(&v.to_string()).is_err());
    }
    let mut v = har_document();
    v["log"]["entries"][0]["time"] = (-1).into();
    assert!(har::inspect(&v.to_string()).is_err());
    let mut v = har_document();
    v["uninterpreted"] = "x".repeat(65537).into();
    assert!(har::inspect(&v.to_string()).is_err());
    let error = har::inspect("{\"private-marker\": broken}").unwrap_err();
    assert!(!error.to_string().contains("private-marker"));
    assert_eq!(error.operation, Some("web.har.inspect"));
}
