use cube_core::{
    forensics::{inspect::FileInfo, signatures},
    investigation::triage,
    operations,
};
use std::io::Cursor;
#[test]
fn recommendations_cite_ordered_evidence_and_available_operations_without_claims() {
    let catalog = operations::catalog();
    for (header, expected) in [
        (b"\x89PNG\r\n\x1a\n".as_slice(), "image.png.inspect"),
        (b"\xff\xd8\xff", "file.exif.inspect"),
        (b"II\x2a\x00", "file.exif.inspect"),
        (b"RIFF\0\0\0\0WAVE", "audio.wave.inspect"),
        (b"unknown", "file.hex.inspect"),
        (b"", "file.strings.extract"),
    ] {
        let result = triage::bytes(header, 12, &catalog).unwrap();
        assert!(result.suggestions.iter().any(|s| s.operation == expected));
        for suggestion in &result.suggestions {
            assert!(catalog.iter().any(|d| d.id == suggestion.operation));
            assert!(suggestion.rule.ends_with(".v1"));
            assert!(!suggestion.uncertainty.is_empty());
            assert!(
                suggestion
                    .evidence
                    .iter()
                    .all(|id| result.facts.iter().any(|f| f.id == *id))
            );
        }
        assert_eq!(result, triage::bytes(header, 12, &catalog).unwrap());
        assert_eq!(
            result,
            triage::reader(&mut Cursor::new(header), 12, &catalog).unwrap()
        );
        assert!(result.human().contains("not structural validation"));
    }
    let result = triage::bytes(b"\x89PNG\r\n\x1a\n", 12, &[]).unwrap();
    assert!(result.suggestions.is_empty());
    assert_eq!(result.facts.len(), 2);
    assert!(
        triage::bytes(b"\x89PN", 12, &catalog)
            .unwrap()
            .suggestions
            .iter()
            .any(|s| s.operation == "file.hex.inspect")
    );
    let mut data = b"\x89PNG\r\n\x1a\n".to_vec();
    data.extend(vec![0; 10000]);
    assert_eq!(
        triage::bytes(&data, 12, &catalog).unwrap().inspected_bytes,
        12
    );
    assert!(triage::bytes(&data, 11, &catalog).is_err());
    assert!(triage::bytes(b"", 13, &catalog).is_err());
    assert!(triage::bytes(b"", 0, &catalog).is_ok());
}
#[test]
fn supplied_file_facts_retain_ambiguity_and_never_reinspect_content() {
    let mut candidates = signatures::detect(b"\x89PNG\r\n\x1a\n");
    candidates.extend(signatures::detect(b"II\x2a\x00"));
    let info = FileInfo {
        size: 12,
        regular_file: true,
        created: None,
        modified: None,
        signatures: candidates,
    };
    let result = triage::from_file_info(&info, &operations::catalog()).unwrap();
    assert_eq!(result.evidence_source, "caller_file_info");
    assert_eq!(result.inspected_bytes, 0);
    assert!(
        result
            .suggestions
            .iter()
            .any(|s| s.operation == "image.png.inspect")
    );
    assert!(
        result
            .suggestions
            .iter()
            .any(|s| s.operation == "file.exif.inspect")
    );
}
