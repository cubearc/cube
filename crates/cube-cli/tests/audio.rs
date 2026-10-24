use std::{path::PathBuf, process::Command};
struct Workspace(PathBuf);
impl Workspace {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cube-audio-{}-{name}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn wav() -> Vec<u8> {
    let mut bytes = b"RIFF".to_vec();
    bytes.extend(40u32.to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(2u16.to_le_bytes());
    bytes.extend(8000u32.to_le_bytes());
    bytes.extend(32000u32.to_le_bytes());
    bytes.extend(4u16.to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(4u32.to_le_bytes());
    bytes.extend([0, 128, 255, 127]);
    bytes
}
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn wave_inspection_has_human_json_and_contextual_error_outputs() {
    let workspace = Workspace::new("inspect");
    let path = workspace.path("sample.wav");
    let input = wav();
    std::fs::write(&path, &input).unwrap();
    let path = path.to_str().unwrap();
    let result = run(&["audio", "inspect", path, "--json"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["frames"], 1);
    assert_eq!(info["format"]["channels"], 2);
    let result = run(&["audio", "inspect", path]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("duration seconds: 1/8000"));
    std::fs::write(path, b"invalid").unwrap();
    let result = run(&["--json-errors", "audio", "inspect", path]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["operation"], "audio.wave.inspect");
}

#[test]
fn sample_outputs_are_explicit_exact_and_never_overwrite() {
    let workspace = Workspace::new("samples");
    let path = workspace.path("input.wav");
    let output = workspace.path("output.pcm");
    let original = wav();
    std::fs::write(&path, &original).unwrap();
    let input = path.to_str().unwrap();
    let result = run(&["audio", "samples", input, "--channels", "1", "--stdout"]);
    assert!(result.status.success());
    assert_eq!(result.stdout, [255, 127]);
    let result = run(&["audio", "samples", input, "--channels", "1,0", "--json"]);
    assert!(result.status.success());
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["values"], serde_json::json!([32767, -32768]));
    assert_eq!(info["data_hex"], "ff7f0080");
    let result = run(&[
        "audio",
        "samples",
        input,
        "--channels",
        "0",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    assert_eq!(std::fs::read(&output).unwrap(), [0, 128]);
    assert_eq!(
        run(&[
            "audio",
            "samples",
            input,
            "--output",
            output.to_str().unwrap()
        ])
        .status
        .code(),
        Some(1)
    );
    assert_eq!(
        run(&["audio", "samples", input, "--output", input])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(
        run(&["audio", "samples", input, "--channels", "0,0", "--stdout"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(run(&["audio", "samples", input]).status.code(), Some(2));
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
