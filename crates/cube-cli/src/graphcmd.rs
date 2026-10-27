//! Explicit graph JSON and query selections; graph algorithms stay in Core.
use clap::{Args, Subcommand};
use cube_core::graph::{self, Graph, Limits};
use std::{error::Error, io::Write};
#[derive(Args)]
pub(crate) struct Common {
    #[command(flatten)]
    input: crate::Input,
    #[arg(long,default_value_t=graph::MAX_WORK)]
    max_work: u64,
    #[arg(long,default_value_t=graph::MAX_MEMORY)]
    max_memory_bytes: usize,
    #[arg(long,default_value_t=graph::MAX_REPORT)]
    max_report_bytes: usize,
}
#[derive(Args)]
pub(crate) struct Route {
    #[command(flatten)]
    common: Common,
    #[arg(long)]
    source: String,
    #[arg(long)]
    target: String,
}
#[derive(Args)]
pub(crate) struct Euler {
    #[command(flatten)]
    common: Common,
    #[arg(long)]
    start: Option<String>,
}
#[derive(Subcommand)]
pub(crate) enum Command {
    Components(Common),
    ShortestPath(Route),
    Cycle(Common),
    EulerTrail(Euler),
}
impl Common {
    fn load(self) -> Result<(Graph, Limits), Box<dyn Error>> {
        let text = self
            .input
            .read_text_limit(graph::MAX_INPUT, "graph.validate")?;
        let limits = Limits {
            max_work: self.max_work,
            max_memory_bytes: self.max_memory_bytes,
            max_report_bytes: self.max_report_bytes,
        };
        Ok((Graph::new(graph::parse(&text)?, limits)?, limits))
    }
}
pub(crate) fn run(command: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Components(a) => {
            let (g, l) = a.load()?;
            serde_json::to_writer(&mut *out, &g.components(l)?)?;
        }
        Command::ShortestPath(a) => {
            let (g, l) = a.common.load()?;
            serde_json::to_writer(&mut *out, &g.shortest_path(&a.source, &a.target, l)?)?;
        }
        Command::Cycle(a) => {
            let (g, l) = a.load()?;
            serde_json::to_writer(&mut *out, &g.cycle(l)?)?;
        }
        Command::EulerTrail(a) => {
            let (g, l) = a.common.load()?;
            serde_json::to_writer(&mut *out, &g.euler_trail(a.start.as_deref(), l)?)?;
        }
    }
    writeln!(out)?;
    Ok(())
}
