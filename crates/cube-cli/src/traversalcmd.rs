//! Explicit traversal configuration and JSON evidence transport.
use cube_core::text::traversal::{self, Direction, Options};
use std::{error::Error, io::Write};
#[derive(clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    input: crate::Input,
    /// Positive fixed/cyclic decimal steps, e.g. 2 or 2,3
    #[arg(long)]
    steps: String,
    /// Required for keyed-skip; Josephus defaults to all source scalars
    #[arg(long)]
    count: Option<usize>,
    #[arg(long)]
    start: Option<u64>,
    #[arg(long, default_value_t = 0)]
    index_base: u8,
    #[arg(long,default_value="forward",value_parser=["forward","reverse"])]
    direction: String,
    #[arg(long)]
    no_wrap: bool,
    #[arg(long,default_value_t=traversal::MAX_WORK)]
    max_work: u64,
    #[arg(long,default_value_t=traversal::MAX_MEMORY)]
    max_memory: usize,
    #[arg(long,default_value_t=cube_core::text::MAX_OUTPUT)]
    max_output: usize,
}
pub(crate) fn run(args: Args, remove: bool, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let op = if remove {
        "text.josephus.extract"
    } else {
        "text.skip.keyed"
    };
    let options = Options {
        start: args.start,
        index_base: args.index_base,
        direction: if args.direction == "forward" {
            Direction::Forward
        } else {
            Direction::Reverse
        },
        wrap: !args.no_wrap,
        max_work: args.max_work,
        max_memory: args.max_memory,
        max_output: args.max_output,
    };
    let steps = traversal::parse_steps(&args.steps).map_err(|mut e| {
        e.operation = Some(op);
        e
    })?;
    let text = args.input.read_text(op)?;
    let report = if remove {
        traversal::josephus(&text, &steps, args.count, &options)?
    } else {
        let count = args.count.ok_or_else(|| {
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::InvalidParameter,
                "keyed-skip requires --count",
            )
            .operation(op)
            .parameter("count")
        })?;
        traversal::keyed(&text, &steps, count, &options)?
    };
    writeln!(output, "{}", serde_json::to_string(&report)?)?;
    Ok(())
}
