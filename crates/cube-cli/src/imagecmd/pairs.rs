use clap::Args;
use cube_core::image::{
    Image,
    pairs::{self, PairOperation},
};
use std::{collections::BTreeSet, error::Error, io::Write, path::PathBuf};
#[derive(Args)]
pub(crate) struct PairArgs {
    /// Explicit ordered paths, without recursion or implicit discovery
    #[arg(num_args=0..=8)]
    paths: Vec<PathBuf>,
    #[arg(long)]
    operation: String,
    #[arg(long, default_value_t = 28)]
    max_pairs: usize,
    #[arg(long, default_value_t = 1000)]
    max_differences: usize,
    /// Show exact counts/bounds after decoding, without pair execution or writes
    #[arg(long)]
    plan: bool,
    #[arg(long, required_unless_present = "plan", conflicts_with = "plan")]
    output_dir: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}
pub(super) fn run(args: PairArgs, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let op = PairOperation::parse(&args.operation)?;
    let mut paths = BTreeSet::new();
    let mut images = Vec::new();
    for path in &args.paths {
        if !paths.insert(path.canonicalize()?) {
            return Err("duplicate canonical image path".into());
        }
        images.push(Image::open(path)?);
    }
    let inputs: Vec<_> = images.iter().collect();
    if args.plan {
        writeln!(
            output,
            "{}",
            serde_json::to_string_pretty(&pairs::plan(
                &inputs,
                op,
                args.max_pairs,
                args.max_differences
            )?)?
        )?;
        return Ok(());
    }
    let report = pairs::execute(&inputs, op, args.max_pairs, args.max_differences)?;
    let manifest = serde_json::to_vec_pretty(&report)?;
    let mut files: Vec<_> = report
        .results
        .into_iter()
        .filter_map(|r| r.name.zip(r.png))
        .collect();
    files.push(("manifest.json".into(), manifest.clone()));
    super::output::write_outputs(&args.output_dir.expect("required destination"), &files)?;
    if args.json {
        output.write_all(&manifest)?;
        writeln!(output)?;
    } else {
        writeln!(
            output,
            "Pairs: {}; successful: {}; failed: {}",
            report.plan.pair_count, report.successes, report.failures
        )?;
    }
    if report.failures > 0 {
        return Err("one or more image pairs failed; inspect manifest.json".into());
    }
    Ok(())
}
