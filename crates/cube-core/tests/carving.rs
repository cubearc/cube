mod common;
use cube_core::{
    errors::ErrorCode,
    forensics::{
        self,
        carving::{self, Selection},
    },
};
fn empty_zip() -> Vec<u8> {
    let mut v = b"PK\x05\x06".to_vec();
    v.extend([0; 18]);
    v
}
#[test]
fn selected_candidates_are_revalidated_and_copied_exactly() {
    let files = common::Workspace::new();
    let png = cube_core::image::Image::from_samples(
        1,
        1,
        cube_core::image::ColorModel::Gray,
        vec![7],
        vec![],
        vec![],
    )
    .unwrap()
    .encode_png()
    .unwrap();
    let gif=b"GIF89a\x01\0\x01\0\x80\0\0\0\0\0\xff\xff\xff\x2c\0\0\0\0\x01\0\x01\0\0\x02\x02\x44\x01\0\x3b".to_vec();
    for (format, data) in [("png", png), ("gif", gif), ("zip", empty_zip())] {
        let source = files.path(&format!("source-{format}"));
        let target = files.path(format!("out-{format}").as_str());
        let mut host = b"host".to_vec();
        host.extend(&data);
        host.extend(b"tail");
        std::fs::write(&source, &host).unwrap();
        let r = carving::carve(
            &source,
            &target,
            Selection::Candidate { offset: 4 },
            data.len() as u64,
        )
        .unwrap();
        assert_eq!(r.offset, 4);
        assert_eq!(r.length, data.len() as u64);
        assert_eq!(r.bytes_written, r.length);
        assert_eq!(r.sha256, cube_core::hashing::sha256(&data));
        assert_eq!(r.selection, "candidate");
        assert_eq!(r.format, Some(format));
        assert_eq!(r.source_size, host.len() as u64);
        assert_eq!(std::fs::read(&target).unwrap(), data);
        assert_eq!(std::fs::read(&source).unwrap(), host);
    }
}
#[test]
fn raw_ranges_and_empty_selections_preserve_exact_bytes_without_validation_claims() {
    let files = common::Workspace::new();
    let source = files.path("source");
    let target = files.path("raw");
    std::fs::write(&source, b"x\0\xff\nend").unwrap();
    let r = carving::carve(
        &source,
        &target,
        Selection::Range {
            offset: 1,
            length: 3,
        },
        3,
    )
    .unwrap();
    assert_eq!(std::fs::read(target).unwrap(), [0, 255, 10]);
    assert_eq!(r.format, None);
    assert_eq!(r.selection, "range");
    assert!(r.evidence.contains("no file-structure validation"));
    let empty = files.path("empty");
    let r = carving::carve(
        &source,
        &empty,
        Selection::Range {
            offset: 7,
            length: 0,
        },
        0,
    )
    .unwrap();
    assert_eq!(r.bytes_written, 0);
    assert_eq!(std::fs::metadata(empty).unwrap().len(), 0);
}
#[test]
fn invalid_or_over_budget_requests_fail_before_destination_creation() {
    let files = common::Workspace::new();
    let source = files.path("source");
    let target = files.path("output");
    std::fs::write(&source, empty_zip()).unwrap();
    for (selection, budget) in [
        (
            Selection::Range {
                offset: 23,
                length: 0,
            },
            100,
        ),
        (
            Selection::Range {
                offset: 0,
                length: 23,
            },
            100,
        ),
        (
            Selection::Range {
                offset: u64::MAX,
                length: 2,
            },
            100,
        ),
        (
            Selection::Range {
                offset: 0,
                length: 2,
            },
            1,
        ),
        (Selection::Candidate { offset: 0 }, 21),
        (Selection::Candidate { offset: 23 }, 100),
        (
            Selection::Range {
                offset: 0,
                length: 0,
            },
            forensics::MAX_COPY_BYTES + 1,
        ),
    ] {
        let error = carving::carve(&source, &target, selection, budget).unwrap_err();
        assert_eq!(error.operation, Some("file.carve"));
        assert!(!target.exists());
    }
    std::fs::write(&source, b"PK\x03\x04false").unwrap();
    assert!(carving::carve(&source, &target, Selection::Candidate { offset: 0 }, 100).is_err());
    assert!(!target.exists());
}
#[test]
fn existing_outputs_source_aliases_and_fresh_source_changes_are_safe() {
    let files = common::Workspace::new();
    let source = files.path("source");
    let target = files.path("existing");
    let data = empty_zip();
    std::fs::write(&source, &data).unwrap();
    std::fs::write(&target, b"keep").unwrap();
    let selected = Selection::Candidate { offset: 0 };
    assert!(carving::carve(&source, &target, selected, 100).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"keep");
    assert!(carving::carve(&source, &source, selected, 100).is_err());
    let hard = files.path("hard");
    std::fs::hard_link(&source, &hard).unwrap();
    assert!(carving::carve(&source, &hard, selected, 100).is_err());
    assert_eq!(std::fs::read(&source).unwrap(), data);
    // A previously inspected candidate is not trusted after source contents change.
    forensics::zip::validate(&mut std::fs::File::open(&source).unwrap(), 0).unwrap();
    std::fs::write(&source, b"PK\x05\x06truncated").unwrap();
    let out = files.path("fresh");
    assert!(carving::carve(&source, &out, selected, 100).is_err());
    assert!(!out.exists());
}
#[cfg(unix)]
#[test]
fn symlink_inputs_and_output_aliases_are_rejected() {
    let files = common::Workspace::new();
    let source = files.path("source");
    let link = files.path("link");
    std::fs::write(&source, empty_zip()).unwrap();
    std::os::unix::fs::symlink(&source, &link).unwrap();
    assert!(carving::carve(&source, &link, Selection::Candidate { offset: 0 }, 100).is_err());
    assert!(
        carving::carve(
            &link,
            &files.path("new"),
            Selection::Candidate { offset: 0 },
            100
        )
        .is_err()
    );
    assert_eq!(std::fs::read(source).unwrap(), empty_zip());
}
#[test]
fn incomplete_zip_boundary_search_requires_an_explicit_range() {
    let files = common::Workspace::new();
    let source = files.path("source");
    let target = files.path("output");
    let mut data = empty_zip();
    data.resize(forensics::zip::MAX_ARCHIVE_BYTES as usize + 1, 0);
    std::fs::write(&source, &data).unwrap();
    let error =
        carving::carve(&source, &target, Selection::Candidate { offset: 0 }, 100).unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceLimit);
    assert!(!target.exists());
    let r = carving::carve(
        &source,
        &target,
        Selection::Range {
            offset: 0,
            length: 22,
        },
        100,
    )
    .unwrap();
    assert_eq!(r.format, None);
    assert_eq!(std::fs::read(target).unwrap(), empty_zip());
}
#[test]
fn embedded_framing_errors_report_absolute_source_offsets() {
    let files = common::Workspace::new();
    let source = files.path("source");
    let target = files.path("output");
    let mut image = cube_core::image::Image::from_samples(
        1,
        1,
        cube_core::image::ColorModel::Gray,
        vec![7],
        vec![],
        vec![],
    )
    .unwrap()
    .encode_png()
    .unwrap();
    image[12] = b'j';
    let mut host = b"prefix!".to_vec();
    host.extend(image);
    std::fs::write(&source, host).unwrap();
    let error =
        carving::carve(&source, &target, Selection::Candidate { offset: 7 }, 1000).unwrap_err();
    assert_eq!(error.offset, Some(15));
    assert_eq!(error.operation, Some("file.carve"));
    assert!(!target.exists());
}
