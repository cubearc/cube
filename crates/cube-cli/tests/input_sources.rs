use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "cube-input-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str, data: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, data).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Err(e) = child.stdin.take().unwrap().write_all(input) {
        assert_eq!(e.kind(), io::ErrorKind::BrokenPipe);
    }
    child.wait_with_output().unwrap()
}
fn success(args: &[&str], input: &[u8], expected: &[u8]) {
    let output = run(args, input);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected, "{args:?}");
    assert!(output.stderr.is_empty());
}
fn failure(args: &[&str], code: &str) -> serde_json::Value {
    let mut full = vec!["--json-errors"];
    full.extend_from_slice(args);
    let output = run(&full, b"must not be used");
    assert!(!output.status.success(), "{args:?}");
    assert!(output.stdout.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(value["error"]["code"], code, "{args:?}: {value}");
    value
}
#[test]
fn inline_text_files_and_stdin_agree_without_inference_or_trimming() {
    let help = run(&["encode", "base64", "--help"], b"");
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for form in ["[TEXT]", "--text", "--file", "-t", "-f", "--input"] {
        assert!(text.contains(form), "{text}");
    }
    let f = Fixture::new();
    let path = f.file("mensagem 空 space.txt", b"Hello");
    let p = path.to_str().unwrap();
    for selector in [
        vec!["Hello"],
        vec!["--text", "Hello"],
        vec!["-t", "Hello"],
        vec!["--file", p],
        vec!["-f", p],
        vec!["--input", p],
    ] {
        let mut args = vec!["encode", "base64"];
        args.extend(selector);
        success(&args, b"ignored stdin", b"SGVsbG8=\n");
    }
    success(&["encode", "base64"], b"Hello", b"SGVsbG8=\n");
    let original = "Olá\nCube🧊";
    let path = f.file("unicode.txt", original.as_bytes());
    for selector in [vec![original], vec!["-f", path.to_str().unwrap()]] {
        let mut args = vec!["text", "reverse"];
        args.extend(selector);
        success(&args, b"", "🧊ebuC\nálO\n".as_bytes());
    }
    success(
        &["text", "reverse"],
        original.as_bytes(),
        "🧊ebuC\nálO\n".as_bytes(),
    );
    success(&["encode", "base64", ""], b"ignored", b"\n");
    success(&["text", "reverse", "--", "-text"], b"", b"txet-\n");
    // A path passed as TEXT remains literal, even when the file exists.
    let encoded = format!("{}\n", cube_core::encoding::base64_encode(p.as_bytes()));
    success(&["encode", "base64", p], b"", encoded.as_bytes());
    assert_eq!(fs::read(path).unwrap(), original.as_bytes());
}
#[test]
fn files_keep_binary_bytes_empty_values_and_utf8_error_offsets() {
    let f = Fixture::new();
    let bytes = [b'A', 255, 0, b'\r', b'\n'];
    let path = f.file("binary.bin", &bytes);
    let p = path.to_str().unwrap();
    success(&["encode", "hex", "-f", p], b"", b"41ff000d0a\n");
    let digest = format!("{}\n", cube_core::hashing::sha256(&bytes));
    success(&["hash", "sha256", "--file", p], b"", digest.as_bytes());
    let e = failure(&["text", "reverse", "-f", p], "invalid_input");
    assert_eq!(e["error"]["operation"], "text.reverse");
    assert_eq!(e["error"]["offset"], 1);
    let encoded = f.file("hex.txt", b"00ff0d0a");
    success(
        &[
            "decode",
            "hex",
            "-f",
            encoded.to_str().unwrap(),
            "--output",
            "raw",
        ],
        b"",
        &[0, 255, 13, 10],
    );
    let empty = f.file("empty.bin", b"");
    success(
        &["encode", "base64", "-f", empty.to_str().unwrap()],
        b"ignored",
        b"\n",
    );
    assert_eq!(fs::read(path).unwrap(), bytes);
}
#[test]
fn conflicting_sources_are_usage_errors_before_any_file_is_opened() {
    for selectors in [
        vec!["Hello", "--text", "World"],
        vec!["Hello", "-f", "missing.bin"],
        vec!["--text", "Hello", "--file", "missing.bin"],
        vec!["-f", "a", "--input", "b"],
    ] {
        let mut args = vec!["encode", "base64"];
        args.extend(selectors);
        failure(&args, "invalid_usage");
    }
    failure(&["encode", "base64", "Hello", "World"], "invalid_usage");
    failure(&["encode", "base64", "--unknown"], "invalid_usage");
}
#[test]
fn regular_file_policy_and_existing_byte_budgets_apply_to_new_sources() {
    let f = Fixture::new();
    let missing = f.0.join("missing.bin");
    failure(&["text", "reverse", "-f", missing.to_str().unwrap()], "io");
    failure(
        &["text", "reverse", "-f", f.0.to_str().unwrap()],
        "invalid_parameter",
    );
    let exact = f.file("exact.txt", &vec![b'A'; cube_core::text::MAX_INPUT]);
    let output = run(&["text", "reverse", "-f", exact.to_str().unwrap()], b"");
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), cube_core::text::MAX_INPUT + 1);
    let excess = f.file("excess.txt", &vec![b'A'; cube_core::text::MAX_INPUT + 1]);
    let e = failure(
        &["text", "reverse", "-f", excess.to_str().unwrap()],
        "resource_limit",
    );
    assert_eq!(e["error"]["operation"], "text.reverse");
    let digits = f.file("digits.txt", &vec![b'0'; 4096]);
    success(
        &["number", "digital-root", "-f", digits.to_str().unwrap()],
        b"",
        b"0\n",
    );
    fs::write(&digits, vec![b'0'; 4097]).unwrap();
    failure(
        &["number", "digital-root", "-f", digits.to_str().unwrap()],
        "resource_limit",
    );
    #[cfg(unix)]
    {
        let link = f.0.join("link.txt");
        std::os::unix::fs::symlink(&exact, &link).unwrap();
        failure(
            &["text", "reverse", "-f", link.to_str().unwrap()],
            "invalid_parameter",
        );
    }
}
#[test]
fn short_keys_work_with_positional_file_and_existing_text_inputs() {
    let f = Fixture::new();
    let path = f.file("message.txt", b"ATTACKATDAWN");
    for selector in [
        vec!["ATTACKATDAWN"],
        vec!["--text", "ATTACKATDAWN"],
        vec!["-f", path.to_str().unwrap()],
    ] {
        let mut args = vec!["cipher", "vigenere", "-k", "LEMON"];
        args.extend(selector);
        success(&args, b"", b"LXFOPVEFRNHR\n");
    }
    let expected = format!(
        "{}\n",
        cube_core::ciphers::beaufort("ATTACK", "LEMON").unwrap()
    );
    success(
        &["cipher", "beaufort", "ATTACK", "-k", "LEMON"],
        b"",
        expected.as_bytes(),
    );
    success(&["binary", "xor", "ab", "-k", "x"], b"", b"191a\n");
}
#[test]
fn pipelines_preserve_explicit_text_and_byte_kinds_and_reject_mixed_selectors() {
    for selector in [vec!["Olá"], vec!["-t", "Olá"], vec!["--text", "Olá"]] {
        let mut args = vec!["pipeline", "run", "--steps", "[]", "--format", "json"];
        args.extend(selector);
        let output = run(&args, b"ignored");
        assert!(output.status.success(), "{:?}", output.stderr);
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            value["output"],
            serde_json::json!({"type":"text","data":"Olá"})
        );
    }
    let f = Fixture::new();
    let path = f.file("raw bytes.bin", &[0, 255]);
    for flag in ["--input", "--file", "-f"] {
        success(
            &[
                "pipeline",
                "run",
                "--steps",
                "[]",
                flag,
                path.to_str().unwrap(),
                "--format",
                "raw",
            ],
            b"",
            &[0, 255],
        );
    }
    for selector in [
        vec!["Hi", "--hex", "00"],
        vec!["Hi", "--value", "{}"],
        vec!["Hi", "--input", "missing.bin"],
        vec!["-t", "Hi", "-f", "missing.bin"],
    ] {
        let mut args = vec!["pipeline", "run", "--steps", "[]"];
        args.extend(selector);
        failure(&args, "invalid_usage");
    }
}
#[test]
fn csv_import_preserves_the_input_alias_and_accepts_the_shared_sources() {
    use cube_core::image::{ColorModel, Image};
    let f = Fixture::new();
    let image = Image::from_bytes(1, 1, ColorModel::Gray, &[42], &[], &[]).unwrap();
    let csv = image.to_csv().unwrap();
    let path = f.file("pixels.csv", csv.as_bytes());
    for (n, selector) in [
        vec!["--input", path.to_str().unwrap()],
        vec!["-f", path.to_str().unwrap()],
        vec![csv.as_str()],
    ]
    .into_iter()
    .enumerate()
    {
        let output = f.0.join(format!("result-{n}.png"));
        let mut args = vec![
            "image",
            "csv",
            "import",
            "--output",
            output.to_str().unwrap(),
        ];
        args.extend(selector);
        let r = run(&args, b"");
        assert!(r.status.success(), "{:?}", r.stderr);
        assert_eq!(Image::open(Path::new(&output)).unwrap().samples(), [42]);
    }
    failure(
        &[
            "image",
            "csv",
            "import",
            "literal",
            "--input",
            path.to_str().unwrap(),
            "--output",
            f.0.join("unused.png").to_str().unwrap(),
        ],
        "invalid_usage",
    );
}
