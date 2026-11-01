#![cfg(unix)]
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn real_cli_uses_only_the_selected_private_context_and_refuses_broad_or_expired_files() {
    let root = std::env::temp_dir().join(format!("cube-attach-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let socket = root.join("control.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600)).unwrap();
    let path = root.join("context.json");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let mut credential = json!({"format":"cubearc.play.context","version":1,"socket":socket,"token":"a".repeat(64),"expires":now+60000});
    fs::write(&path, credential.to_string()).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let attach: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(
            attach,
            json!({"type":"attach","protocol":1,"token":"a".repeat(64)})
        );
        let context = json!({"workspace_id":"11111111-1111-4111-8111-111111111111","exploration_id":"22222222-2222-4222-8222-222222222222","stage_id":"33333333-3333-4333-8333-333333333333","generation":2});
        writeln!(stream,"{}",json!({"type":"context","protocol":1,"cli":"0.18.0","nonce":"b".repeat(32),"context":context,"run":null})).unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["context"], context);
        assert_eq!(
            request["command"],
            json!({"kind":"asset","asset_id":"44444444-4444-4444-8444-444444444444","revision":2})
        );
        writeln!(stream,"{}",json!({"type":"reply","protocol":1,"nonce":"b".repeat(32),"result":{"status":"ok","data":{"text":"authorized source"}}})).unwrap();
    });
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "play",
                "--context",
                path.to_str().unwrap(),
                "asset",
                "show",
                "44444444-4444-4444-8444-444444444444",
                "--revision",
                "2",
            ])
            .output()
            .unwrap()
    };
    let result = run();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["result"]["data"]["text"],
        "authorized source"
    );
    server.join().unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let rejected = run();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("unsafe_context_permissions"));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    credential["expires"] = json!(now - 1);
    fs::write(&path, credential.to_string()).unwrap();
    assert!(String::from_utf8_lossy(&run().stderr).contains("context_expired"));
    fs::remove_dir_all(root).unwrap();
}
