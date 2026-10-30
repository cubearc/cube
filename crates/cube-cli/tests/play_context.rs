use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn cli(args: &[&str], input: Option<Value>) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        writeln!(child.stdin.take().unwrap(), "{input}").unwrap();
    } else {
        drop(child.stdin.take());
    }
    child.wait_with_output().unwrap()
}
#[test]
fn contextual_commands_refuse_missing_context_without_launching_a_runtime() {
    let output = cli(
        &[
            "--json-errors",
            "play",
            "asset",
            "show",
            "00000000-0000-4000-8000-000000000001",
            "--revision",
            "1",
        ],
        None,
    );
    assert_eq!(output.status.code(), Some(1));
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["message"], "context_unavailable");
    assert!(output.stdout.is_empty());
}
#[test]
fn contextual_commands_reject_incompatible_protocol_before_requesting_data() {
    let output = cli(
        &[
            "--json-errors",
            "play",
            "asset",
            "show",
            "00000000-0000-4000-8000-000000000001",
            "--revision",
            "1",
        ],
        Some(
            json!({"type":"context","protocol":99,"cli":"0.18.0","nonce":"0123456789abcdef0123456789abcdef","context":{"workspace_id":"00000000-0000-4000-8000-000000000001","exploration_id":"00000000-0000-4000-8000-000000000002","stage_id":"00000000-0000-4000-8000-000000000003","generation":2},"run":null}),
        ),
    );
    assert_eq!(output.status.code(), Some(1));
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["message"], "protocol_mismatch");
    assert!(output.stdout.is_empty());
}

fn bootstrap() -> Value {
    json!({"type":"context","protocol":1,"cli":"0.18.0","nonce":"0123456789abcdef0123456789abcdef","context":{"workspace_id":"00000000-0000-4000-8000-000000000001","exploration_id":"00000000-0000-4000-8000-000000000002","stage_id":"00000000-0000-4000-8000-000000000003","generation":2},"run":null})
}
#[test]
fn contextual_assets_and_pipeline_commands_exchange_only_closed_machine_frames() {
    use std::io::{BufRead, BufReader, Read};
    for (args, kind) in [
        (
            vec![
                "play",
                "asset",
                "show",
                "00000000-0000-4000-8000-000000000004",
                "--revision",
                "2",
            ],
            "asset",
        ),
        (
            vec![
                "play",
                "pipeline",
                "show",
                "00000000-0000-4000-8000-000000000004",
                "--revision",
                "2",
            ],
            "pipeline",
        ),
        (
            vec![
                "play",
                "run",
                "status",
                "00000000-0000-4000-8000-000000000004",
            ],
            "status",
        ),
        (
            vec![
                "play",
                "run",
                "cancel",
                "00000000-0000-4000-8000-000000000004",
            ],
            "cancel",
        ),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(args)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        writeln!(input, "{}", bootstrap()).unwrap();
        input.flush().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["type"], "request");
        assert_eq!(request["command"]["kind"], kind);
        assert_eq!(request["context"], bootstrap()["context"]);
        let reply = json!({"type":"reply","protocol":1,"nonce":bootstrap()["nonce"],"result":{"status":"ok","data":{"fixture":"known exact result"}}});
        writeln!(input, "{reply}").unwrap();
        drop(input);
        let mut result = String::new();
        output.read_to_string(&mut result).unwrap();
        let outcome = child.wait_with_output().unwrap();
        assert!(outcome.status.success());
        assert!(outcome.stderr.is_empty());
        let result: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["type"], "result");
        assert_eq!(result["result"], reply["result"]);
    }
}
#[test]
fn context_transport_rejects_duplicate_fields_paths_and_capture_side_effects() {
    for init in [
        json!({"type":"context","path":"/private/store"}),
        json!({"type":"context","protocol":1,"cli":"wrong","nonce":"0123456789abcdef0123456789abcdef","context":bootstrap()["context"],"run":null}),
    ] {
        let output = cli(
            &[
                "--json-errors",
                "play",
                "asset",
                "show",
                "00000000-0000-4000-8000-000000000004",
                "--revision",
                "1",
            ],
            Some(init),
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    let output = cli(
        &[
            "--json-errors",
            "--lab-capture=/private/forbidden",
            "play",
            "asset",
            "show",
            "00000000-0000-4000-8000-000000000004",
            "--revision",
            "1",
        ],
        None,
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stderr).unwrap()["error"]["message"],
        "context_options_unavailable"
    );
}

#[test]
fn attached_start_preserves_the_frozen_request_and_refuses_retargeting() {
    use cube_cli::play::{Command as Play, Run, exchange};
    let pipeline = "00000000-0000-4000-8000-000000000004";
    let target = "00000000-0000-4000-8000-000000000005";
    let command = Play::Run {
        command: Run::Start {
            pipeline: pipeline.into(),
            revision: 2,
            target: target.into(),
        },
    };
    let mut init = bootstrap();
    let mut context = init["context"].clone();
    context.as_object_mut().unwrap().remove("generation");
    init["run"] = json!({"pipeline_id":pipeline,"pipeline_revision":2,"target_node_id":target,"context":context,"key":"preserved by client"});
    let reply = json!({"type":"reply","protocol":1,"nonce":init["nonce"],"result":{"status":"ok","data":{"id":"same run"}}});
    let mut output = Vec::new();
    exchange(
        &command,
        &mut format!("{init}\n{reply}\n").as_bytes(),
        &mut output,
    )
    .unwrap();
    let frames = String::from_utf8(output).unwrap();
    let request: Value = serde_json::from_str(frames.lines().next().unwrap()).unwrap();
    assert_eq!(request["command"]["request"], init["run"]);
    for (field, value) in [
        ("pipeline_revision", json!(3)),
        ("target_node_id", json!(pipeline)),
        ("context", json!({})),
    ] {
        let mut changed = init.clone();
        changed["run"][field] = value;
        let mut output = Vec::new();
        assert!(
            exchange(
                &command,
                &mut format!("{changed}\n").as_bytes(),
                &mut output
            )
            .is_err()
        );
        assert!(output.is_empty());
    }
}
#[test]
fn transport_rejects_duplicate_keys_oversized_frames_and_wrong_reply_nonce() {
    use cube_cli::play::{Asset, Command as Play, FRAME_BYTES, exchange};
    let command = Play::Asset {
        command: Asset::Show {
            id: "00000000-0000-4000-8000-000000000004".into(),
            revision: 1,
        },
    };
    let init = bootstrap();
    for input in [
        "{\"type\":\"context\",\"type\":\"context\"}\n".to_owned(),
        format!("{}\n", " ".repeat(FRAME_BYTES)),
        format!(
            "{init}\n{{\"type\":\"reply\",\"protocol\":1,\"nonce\":\"wrong\",\"result\":{{\"status\":\"ok\",\"data\":{{}}}}}}\n"
        ),
    ] {
        let mut output = Vec::new();
        assert!(exchange(&command, &mut input.as_bytes(), &mut output).is_err());
        assert!(
            !String::from_utf8(output)
                .unwrap()
                .contains("\"type\":\"result\"")
        );
    }
}

#[test]
fn run_reply_accepts_transport_sized_evidence_without_changing_pipeline_limits() {
    use cube_cli::play::{Command as Play, Run, exchange};
    let command = Play::Run {
        command: Run::Status {
            id: "00000000-0000-4000-8000-000000000004".into(),
        },
    };
    let init = bootstrap();
    let data = json!({"attempts":vec![0;5000]});
    assert!(cube_core::pipeline::json::parse(&data.to_string()).is_err());
    let reply = json!({"type":"reply","protocol":1,"nonce":init["nonce"],"result":{"status":"ok","data":data}});
    let mut output = Vec::new();
    exchange(
        &command,
        &mut format!("{init}\n{reply}\n").as_bytes(),
        &mut output,
    )
    .unwrap();
    let frames = String::from_utf8(output).unwrap();
    let result: Value = serde_json::from_str(frames.lines().nth(1).unwrap()).unwrap();
    assert_eq!(result["result"]["data"], data);
}
