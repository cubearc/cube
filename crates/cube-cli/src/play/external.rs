//! A user-selected, short-lived native control lease. Never discover ambient stores.
use super::*;
use std::path::Path;
#[cfg(unix)]
use std::{
    fs::{self, OpenOptions},
    io::BufReader,
    os::unix::{
        fs::{FileTypeExt, MetadataExt, OpenOptionsExt},
        net::UnixStream,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(unix)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Credential {
    format: String,
    version: u32,
    socket: String,
    token: String,
    expires: u64,
}

#[cfg(unix)]
fn private(metadata: &fs::Metadata) -> Result<(), OperationError> {
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
        return Err(error("unsafe_context_permissions"));
    }
    Ok(())
}
#[cfg(unix)]
fn credential(path: &Path) -> Result<Credential, OperationError> {
    if !path.is_absolute() {
        return Err(error("invalid_context_file"));
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)
        .map_err(|_| error("context_unavailable"))?;
    let metadata = file.metadata().map_err(|_| error("context_unavailable"))?;
    private(&metadata)?;
    if !metadata.is_file() || metadata.nlink() != 1 || metadata.len() > 4096 {
        return Err(error("invalid_context_file"));
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(4097)
        .read_to_end(&mut bytes)
        .map_err(|_| error("invalid_context_file"))?;
    if bytes.len() > 4096 {
        return Err(error("invalid_context_file"));
    }
    let credential: Credential =
        serde_json::from_slice(&bytes).map_err(|_| error("invalid_context_file"))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| error("context_expired"))?
        .as_millis() as u64;
    if credential.expires <= now || credential.expires - now > 600_000 {
        return Err(error("context_expired"));
    }
    if credential.format != "cubearc.play.context"
        || credential.version != 1
        || credential.token.len() != 64
        || !credential
            .token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !Path::new(&credential.socket).is_absolute()
        || credential.socket.len() > 103
    {
        return Err(error("invalid_context_file"));
    }
    let socket =
        fs::symlink_metadata(&credential.socket).map_err(|_| error("context_unavailable"))?;
    private(&socket)?;
    let parent = Path::new(&credential.socket)
        .parent()
        .ok_or_else(|| error("invalid_context_file"))?;
    let directory = fs::symlink_metadata(parent).map_err(|_| error("context_unavailable"))?;
    private(&directory)?;
    if !socket.file_type().is_socket() || !directory.is_dir() || directory.file_type().is_symlink()
    {
        return Err(error("invalid_context_file"));
    }
    Ok(credential)
}
#[cfg(unix)]
pub fn run(command: &Command, path: &Path) -> Result<(), OperationError> {
    if matches!(command, Command::Session) {
        return Err(error("external_session_unavailable"));
    }
    let credential = credential(path)?;
    let mut stream =
        UnixStream::connect(&credential.socket).map_err(|_| error("context_unavailable"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|()| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| error("transport_closed"))?;
    write(
        &mut stream,
        &serde_json::json!({"type":"attach","protocol":PROTOCOL,"token":credential.token}),
    )?;
    let mut reader = BufReader::new(stream.try_clone().map_err(|_| error("transport_closed"))?);
    let mut reply = round_trip(command, &mut reader, &mut stream)?;
    let success = matches!(reply.result, Outcome::Ok { .. });
    reply.kind = "result".into();
    write(&mut std::io::stdout().lock(), &reply)?;
    if !success {
        return Err(error("request_rejected"));
    }
    Ok(())
}
#[cfg(not(unix))]
pub fn run(_command: &Command, _path: &Path) -> Result<(), OperationError> {
    Err(error("attachment_policy_unavailable"))
}
