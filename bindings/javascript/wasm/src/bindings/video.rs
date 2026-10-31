//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use super::{
    field,
    images::{bytes, channels, image},
};
use cube_core::{
    errors::{ErrorCode, OperationError},
    media::video::{self, audio, comparison, frames},
};
use serde_json::{Value, json};
use std::io::Cursor;
enum Adapter {
    Audio,
    Inspect,
    Compare,
    Frames,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "video.audio.extract" => Adapter::Audio,
        "video.container.inspect" => Adapter::Inspect,
        "video.frames.compare" => Adapter::Compare,
        "video.frames.extract" => Adapter::Frames,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use explicit track IDs and bounded frame selections",
    )
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    let identity = cube_core::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| {
        let mut reader = Cursor::new(super::bytes(&inputs["input"])?);
        Ok(match selected {
            Adapter::Inspect => json!({"output":video::inspect(&mut reader)?}),
            Adapter::Audio => {
                let r = audio::extract(&mut reader, field(&p, "track_id")?)?;
                json!({"output":r,"data":bytes(&r.data)})
            }
            Adapter::Frames => {
                let r = frames::extract(
                    &mut reader,
                    field(&p, "track_id")?,
                    &field::<Vec<usize>>(&p, "indexes")?,
                )?;
                let images: Vec<_> = r.iter().map(|f| image(&f.image)).collect();
                json!({"output":r,"images":images})
            }
            Adapter::Compare => {
                let indexes: Vec<usize> = field(&p, "pairs")?;
                if !indexes.len().is_multiple_of(2) {
                    return Err(invalid().parameter("pairs"));
                }
                let pairs: Vec<_> = indexes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| (p[0], p[1]))
                    .collect();
                json!({"output":comparison::compare(&mut reader,field(&p,"track_id")?,&pairs,channels(&p)?.as_deref(),field(&p,"max_results")?)?})
            }
        })
    })();
    result.map_err(|e: OperationError| e.operation(identity))
}
