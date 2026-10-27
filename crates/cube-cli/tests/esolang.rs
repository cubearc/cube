use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn byte_output_reports_and_explicit_transport_are_consistent() {
    for (code, args, expected) in [
        (
            "++++++++[>++++++++<-]>+.",
            vec!["--output", "text"],
            b"A".to_vec(),
        ),
        (
            ",.",
            vec!["--input-hex", "ff", "--output", "raw"],
            vec![255],
        ),
        ("-.", vec!["--cell-bits", "32"], b"ff\n".to_vec()),
        ("", vec!["--output", "raw"], vec![]),
    ] {
        let mut argv = vec!["esolang", "brainfuck", "run", "--code", code];
        argv.extend(args);
        let r = run(&argv);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        assert_eq!(r.stdout, expected);
    }
    let root = std::env::temp_dir().join(format!("cube-brainfuck-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let code = root.join("source 雪.bf");
    let input = root.join("input.bin");
    std::fs::write(&code, b",.").unwrap();
    std::fs::write(&input, [0, 255]).unwrap();
    let r = run(&[
        "esolang",
        "brainfuck",
        "run",
        "--code-file",
        code.to_str().unwrap(),
        "--input-file",
        input.to_str().unwrap(),
        "--output",
        "raw",
    ]);
    assert!(r.status.success());
    assert_eq!(r.stdout, [0]);
    std::fs::write(&code, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(
        !run(&[
            "esolang",
            "brainfuck",
            "run",
            "--code-file",
            code.to_str().unwrap()
        ])
        .status
        .success()
    );
    std::fs::remove_dir_all(root).unwrap();
    let r = run(&[
        "esolang",
        "brainfuck",
        "report",
        "--code",
        "-.",
        "--cell-bits",
        "16",
        "--tape-len",
        "1",
        "--max-trace-records",
        "2",
        "--max-trace-bytes",
        "1000",
    ]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(v["completed"], true);
    assert_eq!(v["tape"][0]["value"], 65535);
    assert_eq!(v["output"][0], 255);
    assert_eq!(v["trace"].as_array().unwrap().len(), 2);
    let r = run(&[
        "esolang",
        "brainfuck",
        "compare",
        "--code",
        "-.",
        "--widths",
        "32,8,16",
    ]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(v["all_completed"], true);
    assert_eq!(v["equal_completed_outputs"], true);
    assert_eq!(v["runs"][0]["options"]["cell_bits"], 8);
}
#[test]
fn failures_are_typed_and_partial_evidence_never_exits_successfully() {
    for (code, parameter) in [
        ("[", "source"),
        ("<", "pointer"),
        (",", "input"),
        ("+[]", "max_instructions"),
    ] {
        let r = run(&[
            "--json-errors",
            "esolang",
            "brainfuck",
            "run",
            "--code",
            code,
            "--max-instructions",
            "10",
        ]);
        assert!(!r.status.success());
        assert!(r.stdout.is_empty());
        let v: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
        assert_eq!(v["error"]["parameter"], parameter);
    }
    for args in [
        vec!["run", "--code", "-.", "--output", "text"],
        vec!["run", "--code", "+", "--cell-bits", "64"],
        vec!["run", "--code", "+", "--code-file", "bad"],
        vec![
            "run",
            "--code",
            ",",
            "--input-file",
            "bad",
            "--input-hex",
            "00",
        ],
        vec!["run", "--code", "+", "--max-instructions", "-1"],
        vec!["compare", "--code", "", "--widths", "8,8"],
    ] {
        let mut argv = vec!["esolang", "brainfuck"];
        argv.extend(args);
        let r = run(&argv);
        assert!(!r.status.success());
        assert!(r.stdout.is_empty());
    }
    for command in ["report", "compare"] {
        let mut argv = vec![
            "--json-errors",
            "esolang",
            "brainfuck",
            command,
            "--code",
            ".+[]",
            "--max-instructions",
            "3",
        ];
        if command == "compare" {
            argv.extend(["--widths", "8,16", "--total-instructions", "4"]);
        }
        let r = run(&argv);
        assert!(!r.status.success());
        let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
        let _: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
        if command == "report" {
            assert_eq!(v["completed"], false);
            assert_eq!(v["output"][0], 0);
        } else {
            assert_eq!(v["all_completed"], false);
            assert!(v["equal_completed_outputs"].is_null());
        }
    }
    // An unrelated piped stdin is never consumed as source or program input.
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["esolang", "brainfuck", "run", "--code", ",."])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"A").unwrap();
    assert!(!child.wait_with_output().unwrap().status.success());
}
