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
#[test]
fn audio_bits_raw_json_and_exclusive_outputs_match_native_storage() {
    let path = std::env::temp_dir().join(format!("cube-audio-bits-{}.wav", std::process::id()));
    let dest = path.with_extension("bits");
    let mut wav = b"RIFF".to_vec();
    wav.extend(44u32.to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend([1, 0, 1, 0]);
    wav.extend(8000u32.to_le_bytes());
    wav.extend(8000u32.to_le_bytes());
    wav.extend([1, 0, 8, 0]);
    wav.extend(b"data");
    wav.extend(8u32.to_le_bytes());
    wav.extend([1, 0, 0, 1, 0, 1, 1, 0]);
    std::fs::write(&path, &wav).unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["audio", "bits", path.to_str().unwrap()])
            .args(extra)
            .output()
            .unwrap()
    };
    let r = run(&["--stdout"]);
    assert!(r.status.success());
    assert_eq!(r.stdout, [0x96]);
    let r = run(&["--json"]);
    let r: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(r["data_hex"], "96");
    assert!(run(&["--output", dest.to_str().unwrap()]).status.success());
    assert_eq!(std::fs::read(&dest).unwrap(), [0x96]);
    assert!(!run(&["--output", dest.to_str().unwrap()]).status.success());
    assert_eq!(std::fs::read(&path).unwrap(), wav);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(dest).unwrap();
}
#[test]
fn reverse_creates_only_new_wav_and_preserves_sources_and_collisions() {
    let path = std::env::temp_dir().join(format!("cube-audio-reverse-{}.wav", std::process::id()));
    let dest = path.with_extension("reversed.wav");
    let mut wav = b"RIFF".to_vec();
    wav.extend(44u32.to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend([1, 0, 2, 0]);
    wav.extend(8000u32.to_le_bytes());
    wav.extend(16000u32.to_le_bytes());
    wav.extend([2, 0, 8, 0]);
    wav.extend(b"data");
    wav.extend(8u32.to_le_bytes());
    wav.extend([1, 11, 2, 12, 3, 13, 4, 14]);
    std::fs::write(&path, &wav).unwrap();
    let run = |out: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "audio",
                "reverse",
                path.to_str().unwrap(),
                "--output",
                out.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    assert!(run(&dest).status.success());
    let result = std::fs::read(&dest).unwrap();
    let block =
        cube_core::media::pcm::extract(&mut std::io::Cursor::new(&result), &Default::default())
            .unwrap();
    assert_eq!(block.raw(), [4, 14, 3, 13, 2, 12, 1, 11]);
    assert!(!run(&dest).status.success());
    assert!(!run(&path).status.success());
    assert_eq!(std::fs::read(&path).unwrap(), wav);
    assert_eq!(std::fs::read(&dest).unwrap(), result);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(dest).unwrap();
}
#[test]
fn dtmf_json_keeps_nominal_pair_evidence_and_candidate_status() {
    let path = std::env::temp_dir().join(format!("cube-dtmf-{}.wav", std::process::id()));
    let data: Vec<u8> = (0..1600)
        .flat_map(|i| {
            let value = 0.2
                * ((std::f64::consts::TAU * 697.0 * i as f64 / 8000.0).sin()
                    + (std::f64::consts::TAU * 1209.0 * i as f64 / 8000.0).sin());
            ((value * 32768.0).round() as i16).to_le_bytes()
        })
        .collect();
    let mut wav = b"RIFF".to_vec();
    wav.extend((36 + data.len() as u32).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend([1, 0, 1, 0]);
    wav.extend(8000u32.to_le_bytes());
    wav.extend(16000u32.to_le_bytes());
    wav.extend([2, 0, 16, 0]);
    wav.extend(b"data");
    wav.extend((data.len() as u32).to_le_bytes());
    wav.extend(data);
    std::fs::write(&path, &wav).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["audio", "dtmf", path.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(r.status.success(), "{:?}", r.stderr);
    let d: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(d["events"][0]["symbol"], "1");
    assert_eq!(d["events"][0]["status"], "candidate");
    assert_eq!(d["windows"][0]["row_hz"], 697);
    assert_eq!(std::fs::read(&path).unwrap(), wav);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn morse_detection_json_can_be_transcribed_as_a_separate_bounded_command() {
    use std::io::Write;
    use std::process::Stdio;
    let path = std::env::temp_dir().join(format!("cube-morse-{}.wav", std::process::id()));
    let data: Vec<u8> = (0..800)
        .flat_map(|i| {
            ((12000.0 * (std::f64::consts::TAU * 700.0 * i as f64 / 8000.0).sin()).round() as i16)
                .to_le_bytes()
        })
        .collect();
    let mut wav = b"RIFF".to_vec();
    wav.extend((36 + data.len() as u32).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend([1, 0, 1, 0]);
    wav.extend(8000u32.to_le_bytes());
    wav.extend(16000u32.to_le_bytes());
    wav.extend([2, 0, 16, 0]);
    wav.extend(b"data");
    wav.extend((data.len() as u32).to_le_bytes());
    wav.extend(data);
    std::fs::write(&path, &wav).unwrap();
    let detected = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "audio",
            "morse",
            "detect",
            path.to_str().unwrap(),
            "--frequency-hz",
            "700",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(detected.status.success(), "{:?}", detected.stderr);
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["audio", "morse", "transcribe", "--unit-ms", "100", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&detected.stdout)
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    let r: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(r["tokens"], ".");
    assert_eq!(r["text"], "E");
    assert_eq!(std::fs::read(&path).unwrap(), wav);
    std::fs::remove_file(path).unwrap();
}
