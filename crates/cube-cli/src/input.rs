//! CLI source selection only; operations retain their own byte/text semantics.
use clap::Args;
use cube_core::{
    errors::{ErrorCode, OperationError},
    forensics,
};
use std::{
    io::{self, Read},
    path::PathBuf,
};

#[derive(Args, Default)]
#[group(id="data_source", multiple=false, args=["literal","text","file"])]
pub(crate) struct Input {
    /// Literal UTF-8 text; omit all input selectors to read exact stdin bytes
    #[arg(value_name = "TEXT")]
    literal: Option<String>,
    /// Explicit inline text (compatible with the positional TEXT argument)
    #[arg(short = 't', long, value_name = "TEXT")]
    text: Option<String>,
    /// Read exact bytes from a regular file; never infer a path from TEXT
    #[arg(short = 'f', long, visible_alias = "input", value_name = "PATH")]
    file: Option<PathBuf>,
}
impl Input {
    fn read_with_limit(
        self,
        limit: Option<usize>,
        operation: Option<&'static str>,
    ) -> Result<Vec<u8>, OperationError> {
        let context = |mut error: OperationError| {
            error.operation = operation;
            error
        };
        let read = |reader: &mut dyn Read| -> Result<Vec<u8>, OperationError> {
            let mut bytes = Vec::new();
            let max = limit.map_or(u64::MAX, |n| (n as u64).saturating_add(1));
            reader
                .take(max)
                .read_to_end(&mut bytes)
                .map_err(|e| context(OperationError::io(e)))?;
            Ok(bytes)
        };
        let bytes = if let Some(text) = self.literal.or(self.text) {
            text.into_bytes()
        } else if let Some(path) = self.file {
            let mut file = forensics::open_regular(&path).map_err(context)?;
            read(&mut file)?
        } else {
            read(&mut io::stdin().lock())?
        };
        if limit.is_some_and(|max| bytes.len() > max) {
            return Err(context(OperationError::new(
                ErrorCode::ResourceLimit,
                "input exceeds documented operation byte limit",
            )));
        }
        Ok(bytes)
    }
    pub(crate) fn read_bounded(
        self,
        limit: usize,
        operation: &'static str,
    ) -> Result<Vec<u8>, OperationError> {
        self.read_with_limit(Some(limit), Some(operation))
    }
    pub(crate) fn read_text(self, operation: &'static str) -> Result<String, OperationError> {
        self.read_text_limit(cube_core::text::MAX_INPUT, operation)
    }
    pub(crate) fn read_text_limit(
        self,
        limit: usize,
        operation: &'static str,
    ) -> Result<String, OperationError> {
        let bytes = self.read_bounded(limit, operation)?;
        String::from_utf8(bytes).map_err(|e| {
            OperationError::new(ErrorCode::InvalidInput, "input must be valid UTF-8")
                .operation(operation)
                .offset(e.utf8_error().valid_up_to() as u64)
                .caused_by(e)
        })
    }
    // Legacy transformations retain their existing unrestricted byte input API.
    pub(crate) fn read(self) -> Result<Vec<u8>, OperationError> {
        self.read_with_limit(None, None)
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    #[test]
    fn all_commands_have_consistent_argument_and_conflict_definitions() {
        super::super::Cli::command().debug_assert();
    }
}
