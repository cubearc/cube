use cube_core::forensics::inspect::artifact_info;

#[test]
fn byte_artifacts_report_evidence_without_fabricating_filesystem_facts() {
    let actual = serde_json::to_value(artifact_info(b"GIF89a\x00")).unwrap();
    assert_eq!(actual["size"], 7);
    assert_eq!(actual["source_kind"], "artifact");
    assert!(actual["regular_file"].is_null());
    assert!(actual["modified"].is_null());
    assert!(actual["created"].is_null());
    assert!(!actual["signatures"].as_array().unwrap().is_empty());
}
