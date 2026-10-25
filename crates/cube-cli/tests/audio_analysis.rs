use std::process::Command;
#[test]
fn spectrum_json_reuses_pcm_channels_and_exact_frequency_axes() {
    let path = std::env::temp_dir().join(format!("cube-spectrum-{}.wav", std::process::id()));
    let mut wav = b"RIFF".to_vec();
    wav.extend(52u32.to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend([1, 0, 1, 0]);
    wav.extend(8000u32.to_le_bytes());
    wav.extend(16000u32.to_le_bytes());
    wav.extend([2, 0, 16, 0]);
    wav.extend(b"data");
    wav.extend(16u32.to_le_bytes());
    wav.extend([0u8; 16]);
    std::fs::write(&path, &wav).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "audio",
            "spectrum",
            path.to_str().unwrap(),
            "--fft-size",
            "8",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(r.status.success(), "{:?}", r.stderr);
    let r: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(r["frequencies_hz"][1], 1000.0);
    assert_eq!(r["amplitudes"][0], serde_json::json!(vec![0.0; 5]));
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "audio",
            "spectrum",
            path.to_str().unwrap(),
            "--fft-size",
            "9",
        ])
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), wav);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn spectrogram_json_exposes_short_padding_and_preflight_errors() {
    let path = std::env::temp_dir().join(format!("cube-spectrogram-{}.wav", std::process::id()));
    let mut wav = b"RIFF".to_vec();
    wav.extend(52u32.to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend([1, 0, 1, 0]);
    wav.extend(8000u32.to_le_bytes());
    wav.extend(16000u32.to_le_bytes());
    wav.extend([2, 0, 16, 0]);
    wav.extend(b"data");
    wav.extend(16u32.to_le_bytes());
    wav.extend([0u8; 16]);
    std::fs::write(&path, &wav).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "audio",
            "spectrogram",
            path.to_str().unwrap(),
            "--fft-size",
            "8",
            "--hop-size",
            "4",
            "--final-window",
            "pad",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(r.status.success(), "{:?}", r.stderr);
    let r: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(r["frame_starts"], serde_json::json!([0, 4]));
    assert_eq!(r["valid_frames"], serde_json::json!([8, 4]));
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "audio",
            "spectrogram",
            path.to_str().unwrap(),
            "--hop-size",
            "0",
        ])
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    std::fs::remove_file(path).unwrap();
}
