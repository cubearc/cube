//! Closed contextual commands over an inherited session or an explicitly issued private lease.
//!
//! This is a control client. Play owns authorization, planning, persistence and
//! restricted operation workers. No database, filesystem, shell or network API
//! is part of this protocol. External leases and native PTYs share its scoped requests.
use clap::Subcommand;
use cube_core::errors::{ErrorCode, OperationError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, IsTerminal, Read, Write};

mod external;
mod session;
pub use external::run as run_external;

pub const PROTOCOL: u32 = 1;
pub const FRAME_BYTES: usize = 262_144;
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Open the app-attached contextual terminal (requires native inherited channels)
    Session,
    /// Inspect an exact Asset revision in the attached Stage
    Asset {
        #[command(subcommand)]
        command: Asset,
    },
    /// Inspect a saved Workspace Pipeline revision
    Pipeline {
        #[command(subcommand)]
        command: Pipeline,
    },
    /// Start, inspect or cancel an owned PipelineRun
    Run {
        #[command(subcommand)]
        command: Run,
    },
}
#[derive(Subcommand, Debug)]
pub enum Asset {
    Show {
        #[arg(value_parser=identity)]
        id: String,
        #[arg(long,value_parser=clap::value_parser!(u64).range(1..))]
        revision: u64,
    },
}
#[derive(Subcommand, Debug)]
pub enum Pipeline {
    Show {
        #[arg(value_parser=identity)]
        id: String,
        #[arg(long,value_parser=clap::value_parser!(u64).range(1..))]
        revision: u64,
    },
}
#[derive(Subcommand, Debug)]
pub enum Run {
    /// Execute the attached Trail's exact selected Pipeline and target
    Start {
        #[arg(long,value_parser=identity)]
        pipeline: String,
        #[arg(long,value_parser=clap::value_parser!(u64).range(1..))]
        revision: u64,
        #[arg(long,value_parser=identity)]
        target: String,
    },
    Status {
        #[arg(value_parser=identity)]
        id: String,
    },
    Cancel {
        #[arg(value_parser=identity)]
        id: String,
    },
}
fn identity(value: &str) -> Result<String, String> {
    let valid = value.len() == 36
        && value.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        });
    if valid {
        Ok(value.to_owned())
    } else {
        Err("expected a canonical UUID".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub workspace_id: String,
    pub exploration_id: String,
    pub stage_id: String,
    pub generation: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bootstrap {
    #[serde(rename = "type")]
    pub kind: String,
    pub protocol: u32,
    pub cli: String,
    pub nonce: String,
    pub context: Context,
    pub run: Option<Value>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Asset { asset_id: String, revision: u64 },
    Pipeline { pipeline_id: String, revision: u64 },
    Start { request: Value },
    Status { run_id: String },
    Cancel { run_id: String },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(rename = "type")]
    pub kind: String,
    pub protocol: u32,
    pub nonce: String,
    pub context: Context,
    pub command: Action,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Ok { data: Value },
    Rejected { code: String },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    #[serde(rename = "type")]
    pub kind: String,
    pub protocol: u32,
    pub nonce: String,
    pub result: Outcome,
}
fn error(message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}
fn read(reader: &mut impl BufRead) -> Result<Value, OperationError> {
    let mut bytes = Vec::new();
    let size = reader
        .take(FRAME_BYTES as u64 + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| error("transport_closed"))?;
    if size == 0 {
        return Err(error("context_unavailable"));
    }
    if size > FRAME_BYTES || bytes.last() != Some(&b'\n') {
        return Err(error("invalid_frame"));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| error("invalid_frame"))?;
    cube_core::pipeline::json::parse_with_limits(text, FRAME_BYTES, 64, FRAME_BYTES)
        .map_err(|_| error("invalid_frame"))
}
fn write(writer: &mut impl Write, value: &impl Serialize) -> Result<(), OperationError> {
    let mut bytes = serde_json::to_vec(value).map_err(|_| error("invalid_frame"))?;
    bytes.push(b'\n');
    if bytes.len() > FRAME_BYTES {
        return Err(error("output_limit"));
    }
    writer
        .write_all(&bytes)
        .and_then(|()| writer.flush())
        .map_err(|_| error("transport_closed"))
}
fn authorize(bootstrap: &Bootstrap) -> Result<(), OperationError> {
    if bootstrap.kind != "context" || bootstrap.protocol != PROTOCOL || bootstrap.cli != CLI_VERSION
    {
        return Err(error("protocol_mismatch"));
    }
    if bootstrap.nonce.len() != 32
        || !bootstrap.nonce.bytes().all(|b| b.is_ascii_hexdigit())
        || bootstrap.context.generation == 0
    {
        return Err(error("invalid_context"));
    }
    for id in [
        &bootstrap.context.workspace_id,
        &bootstrap.context.exploration_id,
        &bootstrap.context.stage_id,
    ] {
        identity(id).map_err(|_| error("invalid_context"))?;
    }
    Ok(())
}
fn action(command: &Command, bootstrap: &Bootstrap) -> Result<Action, OperationError> {
    Ok(match command {
        Command::Session => return Err(error("invalid_command")),
        Command::Asset {
            command: Asset::Show { id, revision },
        } => Action::Asset {
            asset_id: id.clone(),
            revision: *revision,
        },
        Command::Pipeline {
            command: Pipeline::Show { id, revision },
        } => Action::Pipeline {
            pipeline_id: id.clone(),
            revision: *revision,
        },
        Command::Run {
            command: Run::Status { id },
        } => Action::Status { run_id: id.clone() },
        Command::Run {
            command: Run::Cancel { id },
        } => Action::Cancel { run_id: id.clone() },
        Command::Run {
            command:
                Run::Start {
                    pipeline,
                    revision,
                    target,
                },
        } => {
            let request = bootstrap.run.as_ref().ok_or(error("selection_required"))?;
            if request["pipeline_id"] != *pipeline
                || request["pipeline_revision"] != *revision
                || request["target_node_id"] != *target
            {
                return Err(error("selection_mismatch"));
            }
            for (key, id) in [
                ("workspace_id", &bootstrap.context.workspace_id),
                ("exploration_id", &bootstrap.context.exploration_id),
                ("stage_id", &bootstrap.context.stage_id),
            ] {
                if request["context"][key] != *id {
                    return Err(error("context_mismatch"));
                }
            }
            Action::Start {
                request: request.clone(),
            }
        }
    })
}
/// Exchange one closed request. The trusted parent supplies the exact context and
/// lower limits, authenticates the reply and owns process timeout/cleanup.
fn round_trip(
    command: &Command,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Reply, OperationError> {
    let bootstrap: Bootstrap =
        serde_json::from_value(read(reader)?).map_err(|_| error("invalid_context"))?;
    authorize(&bootstrap)?;
    let request = Request {
        kind: "request".into(),
        protocol: PROTOCOL,
        nonce: bootstrap.nonce.clone(),
        context: bootstrap.context.clone(),
        command: action(command, &bootstrap)?,
    };
    write(writer, &request)?;
    let reply: Reply = serde_json::from_value(read(reader)?).map_err(|_| error("invalid_reply"))?;
    if reply.kind != "reply" || reply.protocol != PROTOCOL || reply.nonce != bootstrap.nonce {
        return Err(error("protocol_mismatch"));
    }
    Ok(reply)
}
pub fn exchange(
    command: &Command,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<(), OperationError> {
    let mut reply = round_trip(command, reader, writer)?;
    let success = matches!(reply.result, Outcome::Ok { .. });
    reply.kind = "result".into();
    write(writer, &reply)?;
    if !success {
        return Err(error("request_rejected"));
    }
    Ok(())
}
/// Enter an authorized inherited PTY/control session with a host startup policy.
/// The owner reader starts before the policy runs, and no interactive command is
/// read until it succeeds. Native hosts can then deny further thread creation.
/// Missing inherited authority returns before invoking the policy.
pub fn run_session_with_policy(
    policy: impl FnOnce() -> Result<(), OperationError>,
) -> Result<(), OperationError> {
    session::run(policy)
}

/// Contextual commands have no discovery or launcher fallback. A terminal alone
/// does not supply an application-authorized inherited session.
pub fn run(command: &Command) -> Result<(), OperationError> {
    if matches!(command, Command::Session) {
        return run_session_with_policy(|| Ok(()));
    }
    if std::io::stdin().is_terminal() {
        return Err(error("context_unavailable"));
    }
    exchange(
        command,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    )
}
