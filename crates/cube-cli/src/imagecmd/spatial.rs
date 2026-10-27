//! Explicit selected region, one channel, ordered lags; Core computes moments.
use super::*;
use image::spatial as s;
#[derive(Args)]
pub(crate) struct Options {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    region: Region,
    #[arg(long)]
    channel: String,
    /// JSON array [{dx,dy},...], signed i32 offsets, no wrapping
    #[arg(long)]
    lags: String,
    #[arg(long)]
    expand_palette: bool,
    #[arg(long,default_value_t=s::MAX_WORK)]
    max_work: usize,
    #[arg(long,default_value_t=s::MAX_MEMORY)]
    max_memory_bytes: usize,
    #[arg(long,default_value_t=s::MAX_REPORT)]
    max_report_bytes: usize,
}
pub(crate) fn run(args: Options, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let channel = Channel::parse(&args.channel)?;
    let lags = s::parse_lags(&args.lags)?;
    let image = Image::open(&args.input)?;
    let view = image.view(args.expand_palette)?;
    let result = view.autocorrelation(
        args.region,
        channel,
        &lags,
        s::Limits {
            max_work: args.max_work,
            max_memory_bytes: args.max_memory_bytes,
            max_report_bytes: args.max_report_bytes,
        },
    )?;
    serde_json::to_writer(&mut *out, &result)?;
    writeln!(out)?;
    Ok(())
}
