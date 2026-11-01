//! Interactive presentation over a PTY; all domain actions stay on the private socket.
use super::*;
#[cfg(any(unix, test))]
use clap::Parser;

#[cfg(any(unix, test))]
#[derive(Parser)]
#[command(name = "cube play", disable_version_flag = true)]
struct Line {
    #[command(subcommand)]
    command: Command,
}

#[cfg(any(unix, test))]
fn command(line: &str) -> Result<Command, String> {
    if line
        .bytes()
        .any(|b| matches!(b, b'|' | b'>' | b'<' | b'$' | b'`' | b';' | b'&'))
    {
        return Err("Shell expressions are unavailable. Use help for contextual commands.".into());
    }
    let words = line.split_ascii_whitespace().collect::<Vec<_>>();
    let words = words.strip_prefix(&["cube", "play"]).unwrap_or(&words);
    Line::try_parse_from(std::iter::once("cube play").chain(words.iter().copied()))
        .map(|line| line.command)
        .map_err(|e| e.to_string())
}

/// The terminal accepts ASCII command syntax and UUIDs; it stores no history.
#[cfg(any(unix, test))]
fn line(input: &mut impl Read, output: &mut impl Write) -> Result<Option<String>, OperationError> {
    let mut bytes = Vec::new();
    let mut escape = false;
    output
        .write_all(b"cube play> ")
        .map_err(|_| error("terminal_closed"))?;
    loop {
        output.flush().map_err(|_| error("terminal_closed"))?;
        let mut byte = [0];
        if input
            .read(&mut byte)
            .map_err(|_| error("terminal_closed"))?
            == 0
        {
            return Ok(None);
        }
        let b = byte[0];
        if escape {
            escape = !(b.is_ascii_alphabetic() || b == b'~');
            continue;
        }
        match b {
            27 => escape = true,
            4 if bytes.is_empty() => return Ok(None),
            3 => {
                bytes.clear();
                output
                    .write_all(b"^C\r\ncube play> ")
                    .map_err(|_| error("terminal_closed"))?;
            }
            b'\r' | b'\n' => {
                output
                    .write_all(b"\r\n")
                    .map_err(|_| error("terminal_closed"))?;
                return String::from_utf8(bytes)
                    .map(Some)
                    .map_err(|_| error("invalid_command"));
            }
            8 | 127 if !bytes.is_empty() => {
                bytes.pop();
                output
                    .write_all(b"\x08 \x08")
                    .map_err(|_| error("terminal_closed"))?;
            }
            32..=126 if bytes.len() == 1024 => return Err(error("input_limit")),
            32..=126 => {
                bytes.push(b);
                output
                    .write_all(&[b])
                    .map_err(|_| error("terminal_closed"))?;
            }
            _ => (),
        }
    }
}

#[cfg(unix)]
fn display(output: &mut impl Write, text: &str) -> Result<(), OperationError> {
    let text = text.replace('\n', "\r\n");
    let end = text
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|i| *i <= 16384)
        .last()
        .filter(|_| text.len() > 16384)
        .unwrap_or(text.len());
    output
        .write_all(&text.as_bytes()[..end])
        .map_err(|_| error("terminal_closed"))?;
    if end < text.len() {
        output
            .write_all(b"\r\n[Display truncated; inspect the saved Run in Play.]")
            .map_err(|_| error("terminal_closed"))?;
    }
    output
        .write_all(b"\r\n")
        .and_then(|()| output.flush())
        .map_err(|_| error("terminal_closed"))
}

#[cfg(any(unix, test))]
fn result_text(data: &Value) -> Result<String, OperationError> {
    let pretty = |value| serde_json::to_string_pretty(value).map_err(|_| error("invalid_reply"));
    if data.get("caller").is_none() {
        return pretty(data);
    }
    let id = data["id"].as_str().ok_or_else(|| error("invalid_reply"))?;
    let status = data["status"]
        .as_str()
        .ok_or_else(|| error("invalid_reply"))?;
    let mut text = format!("Run {id}: {status}");
    if let Some(result) = data.get("result").filter(|r| !r.is_null()) {
        text.push_str(&format!("\n{}", pretty(result)?));
    }
    if matches!(status, "queued" | "running") {
        text.push_str(&format!(
            "\nInspect: run status {id}\nCancel: run cancel {id}"
        ));
    }
    Ok(text)
}

#[cfg(unix)]
pub(super) fn run(
    policy: impl FnOnce() -> Result<(), OperationError>,
) -> Result<(), OperationError> {
    use std::{io::BufReader, os::unix::net::UnixStream, sync::mpsc, time::Duration};
    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        return Err(error("context_unavailable"));
    }
    let fd = rustix::io::dup(std::io::stdout()).map_err(|_| error("context_unavailable"))?;
    let mut control = UnixStream::from(fd);
    control
        .peer_addr()
        .map_err(|_| error("context_unavailable"))?;
    let mut reader = BufReader::new(control.try_clone().map_err(|_| error("transport_closed"))?);
    let (send, receive) = mpsc::sync_channel(2);
    std::thread::Builder::new()
        .name("terminal-owner".into())
        .spawn(move || {
            loop {
                let Ok(frame) = read(&mut reader) else {
                    std::process::exit(1)
                };
                if send.try_send(frame).is_err() {
                    std::process::exit(1);
                }
            }
        })
        .map_err(|_| error("owner_watch_unavailable"))?;
    policy()?;
    let next = || {
        receive
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| error("transport_closed"))
    };
    let bootstrap = || -> Result<Bootstrap, OperationError> {
        let value: Bootstrap =
            serde_json::from_value(next()?).map_err(|_| error("invalid_context"))?;
        authorize(&value)?;
        Ok(value)
    };
    let mut attached = bootstrap()?;
    let context = attached.context.clone();
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stderr().lock();
    display(
        &mut output,
        "Cube contextual terminal. Type help, selected, size or exit.\nCtrl+C clears input. Runs continue until completed or explicitly cancelled.",
    )?;
    for _ in 0..64 {
        let Some(text) = line(&mut input, &mut output)? else {
            return Ok(());
        };
        let text = text.trim();
        match text {
            "exit" => return Ok(()),
            "" => continue,
            "help" => {
                display(
                    &mut output,
                    "asset show ID --revision N\npipeline show ID --revision N\nrun start --pipeline ID --revision N --target ID\nrun status ID\nrun cancel ID\nselected: run the attached Pipeline selection\nsize: inspect PTY columns and rows\nexit: close this terminal",
                )?;
                continue;
            }
            "size" => {
                let size = rustix::termios::tcgetwinsize(std::io::stdin())
                    .map_err(|_| error("terminal_closed"))?;
                display(
                    &mut output,
                    &format!("{} columns, {} rows", size.ws_col, size.ws_row),
                )?;
                continue;
            }
            _ => (),
        }
        let chosen = if text == "selected" {
            attached
                .run
                .as_ref()
                .map(|request| Command::Run {
                    command: Run::Start {
                        pipeline: request["pipeline_id"].as_str().unwrap_or_default().into(),
                        revision: request["pipeline_revision"].as_u64().unwrap_or_default(),
                        target: request["target_node_id"]
                            .as_str()
                            .unwrap_or_default()
                            .into(),
                    },
                })
                .ok_or_else(|| {
                    "No Pipeline selected. Open a Trail and reconnect the terminal.".to_owned()
                })
        } else {
            command(text)
        };
        let action = match chosen.and_then(|c| action(&c, &attached).map_err(|e| e.to_string())) {
            Ok(value) => value,
            Err(message) => {
                display(&mut output, &message)?;
                continue;
            }
        };
        write(
            &mut control,
            &Request {
                kind: "request".into(),
                protocol: PROTOCOL,
                nonce: attached.nonce.clone(),
                context: context.clone(),
                command: action,
            },
        )?;
        let mut reply: Reply =
            serde_json::from_value(next()?).map_err(|_| error("invalid_reply"))?;
        if reply.kind != "reply" || reply.protocol != PROTOCOL || reply.nonce != attached.nonce {
            return Err(error("protocol_mismatch"));
        }
        let text = match &reply.result {
            Outcome::Ok { data } => result_text(data)?,
            Outcome::Rejected { code } => format!(
                "Request refused: {}",
                serde_json::to_string(code).map_err(|_| error("invalid_reply"))?
            ),
        };
        reply.kind = "result".into();
        write(&mut control, &reply)?;
        display(&mut output, &text)?;
        attached = bootstrap()?;
        if attached.context != context {
            return Err(error("context_mismatch"));
        }
    }
    display(
        &mut output,
        "Session command limit reached. Reconnect to continue.",
    )
}

#[cfg(not(unix))]
pub(super) fn run(
    _policy: impl FnOnce() -> Result<(), OperationError>,
) -> Result<(), OperationError> {
    Err(error("native_policy_unavailable"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_output_shows_state_and_evidence_without_dumping_the_plan() {
        let data = serde_json::json!({"id":"abc","caller":"cli","status":"completed", "plan":{"private":"configuration"},"result":{"outputs":{"text":"Hello"}}});
        let text = result_text(&data).unwrap();
        assert!(text.contains("Run abc: completed"));
        assert!(text.contains("Hello"));
        assert!(!text.contains("configuration"));
    }
    #[test]
    fn syntax_never_dispatches_a_shell_or_launcher() {
        for text in [
            "ls",
            "cube play /tmp",
            "session",
            "$(whoami)",
            "asset show a | cat",
        ] {
            let result = command(text);
            assert!(result.is_err() || matches!(result, Ok(Command::Session)));
        }
        assert!(command("cube play run status 00000000-0000-0000-0000-000000000001").is_ok());
    }
    #[test]
    fn input_editing_is_bounded_and_control_c_only_clears_the_draft() {
        let mut output = Vec::new();
        assert_eq!(
            line(&mut &b"old\x03helx\x7flo\r"[..], &mut output)
                .unwrap()
                .as_deref(),
            Some("hello")
        );
        assert_eq!(line(&mut &b"\x04"[..], &mut output).unwrap(), None);
        assert!(
            line(
                &mut [vec![b'x'; 1025], vec![b'\r']].concat().as_slice(),
                &mut output
            )
            .is_err()
        );
    }
    #[test]
    fn a_missing_inherited_terminal_never_applies_a_host_policy() {
        let invoked = std::cell::Cell::new(false);
        let result = super::super::run_session_with_policy(|| {
            invoked.set(true);
            Err(error("test_policy_denied"))
        });
        assert!(result.is_err());
        assert!(!invoked.get());
    }
}
