use std::{path::PathBuf, process::Command};
struct Files(PathBuf);
impl Files {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("cube-cli-{}-{name}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn file_inspection_and_safe_raw_extraction() {
    let files = Files::new("basic");
    let source = files.path("misleading.png");
    let target = files.path("target");
    let bytes = b"%PDF-1.7\0\xffPK\x03\x04";
    std::fs::write(&source, bytes).unwrap();
    let path = source.to_str().unwrap();
    for args in [
        vec!["inspect", path, "--json"],
        vec!["file", "signature", path, "--json"],
        vec!["file", "embedded", path, "--json"],
    ] {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(output.stderr.is_empty());
    }
    let output = run(&[
        "file", "extract", path, "--offset", "8", "--length", "2", "--stdout",
    ]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\0\xff");
    let output = run(&[
        "file",
        "extract",
        path,
        "--offset",
        "8",
        "--length",
        "2",
        "--output",
        target.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(std::fs::read(&target).unwrap(), b"\0\xff");
    let output = run(&[
        "--json-errors",
        "file",
        "extract",
        path,
        "--offset",
        "0",
        "--length",
        "1",
        "--output",
        target.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "io");
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    let output = run(&["file", "extract", path, "--offset", "0", "--length", "1"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn hex_strings_and_compare_have_structured_and_human_output() {
    let files = Files::new("ranges");
    let first = files.path("first");
    let second = files.path("second");
    std::fs::write(&first, b"\0hello\xfftail").unwrap();
    std::fs::write(&second, b"\0Hello").unwrap();
    let a = first.to_str().unwrap();
    let b = second.to_str().unwrap();
    for args in [
        vec!["file", "hex", a, "--json"],
        vec!["file", "strings", a, "--json"],
        vec!["file", "compare", a, b, "--json"],
    ] {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    }
    let output = run(&[
        "file", "hex", a, "--offset", "5", "--length", "2", "--width", "2",
    ]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"00000005  6f ff  |o.|\n");
    let output = run(&["file", "strings", a]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("00000001  5  hello"));
    let output = run(&["file", "compare", a, b]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("EOF"));
}

#[test]
fn exif_and_png_boundaries_have_safe_errors_and_json() {
    let files = Files::new("formats");
    let path = files.path("input.tiff");
    let tiff = b"II\x2a\0\x08\0\0\0\x01\0\x0f\x01\x02\0\x04\0\0\0Cube\0\0\0\0";
    std::fs::write(&path, tiff).unwrap();
    let name = path.to_str().unwrap();
    let output = run(&["file", "exif", name, "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["tags"][0]["name"], "Make");
    let output = run(&["file", "exif", name]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Cube"));
    let output = run(&["--json-errors", "file", "appended", name]);
    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(json["error"]["code"], "unsupported_format");
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    for (kind, payload) in [
        (&b"IHDR"[..], &[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0][..]),
        (b"IDAT", &[][..]),
        (b"IEND", &[][..]),
    ] {
        png.extend((payload.len() as u32).to_be_bytes());
        png.extend(kind);
        png.extend(payload);
        let mut crcdata = kind.to_vec();
        crcdata.extend(payload);
        let crc = u32::from_str_radix(&cube_core::hashing::crc32(&crcdata), 16).unwrap();
        png.extend(crc.to_be_bytes());
    }
    png.extend(b"trailer");
    std::fs::write(&path, png).unwrap();
    let output = run(&["file", "appended", name, "--json"]);
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["trailing_bytes"], 7);
    let output = run(&["file", "appended", name]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Trailing bytes: 7"));
}

#[test]
fn human_file_information_names_timestamp_units() {
    let files = Files::new("timestamp");
    let source = files.path("input");
    std::fs::write(&source, b"data").unwrap();
    let output = run(&["inspect", source.to_str().unwrap()]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Size: 4 bytes"));
    assert!(text.contains("Unix epoch") || text.contains("unavailable"));
    assert!(!text.contains("Some("));
}
#[test]
fn appended_gif_reports_exact_bounds_without_modifying_source() {
    let files = Files::new("gif-boundary");
    let path = files.path("input");
    let gif=b"GIF89a\x01\0\x01\0\x80\0\0\0\0\0\xff\xff\xff\x2c\0\0\0\0\x01\0\x01\0\0\x02\x02\x44\x01\0\x3btail";
    std::fs::write(&path, gif).unwrap();
    let r = run(&["file", "appended", path.to_str().unwrap(), "--json"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let json: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(json["format"], "gif");
    assert_eq!(json["logical_end"], gif.len() - 4);
    assert_eq!(json["trailing_bytes"], 4);
    assert_eq!(std::fs::read(&path).unwrap(), gif);
}
#[test]
fn zip_candidate_validation_is_distinct_from_signature_detection() {
    let files = Files::new("zip-validation");
    let path = files.path("host");
    let mut bytes = b"hostPK\x05\x06".to_vec();
    bytes.extend([0; 18]);
    bytes.extend(b"tail");
    std::fs::write(&path, &bytes).unwrap();
    let r = run(&[
        "file",
        "zip",
        path.to_str().unwrap(),
        "--offset",
        "4",
        "--json",
    ]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["length"], 22);
    assert_eq!(report["trailing_bytes"], 4);
    assert_eq!(report["payload_verified"], false);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    std::fs::write(&path, b"PK\x03\x04false").unwrap();
    let r = run(&["file", "zip", path.to_str().unwrap(), "--json"]);
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
}
