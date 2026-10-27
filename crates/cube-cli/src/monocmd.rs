//! Explicit source/transport adaptation; all mappings and validation live in Core.
use crate::Input;
use clap::Args;
use cube_core::{
    ciphers::mono::{
        self, alphabet,
        checkerboard::{Profile, Table},
        grandpre, homophonic, params,
    },
    errors::{ErrorCode, OperationError},
};
use std::{error::Error, io::Read, path::PathBuf};
#[derive(Args)]
pub(crate) struct Substitution {
    #[command(flatten)]
    input: Input,
    #[arg(
        long,
        required_unless_present = "definition",
        conflicts_with = "definition"
    )]
    source: Option<String>,
    /// Strict JSON source plus target/keyword and policy; supports escaped NUL
    #[arg(long,conflicts_with_all=["source","target","keyword","unmapped"])]
    definition: Option<PathBuf>,
    #[arg(long, conflicts_with = "keyword", required_unless_present_any = ["keyword","definition"])]
    target: Option<String>,
    #[arg(long, conflicts_with = "target", required_unless_present_any = ["target","definition"])]
    keyword: Option<String>,
    #[arg(long, default_value = "reject")]
    unmapped: String,
    #[arg(long)]
    decode: bool,
}
pub(crate) fn substitution(args: Substitution) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.substitution.decode"
    } else {
        "cipher.substitution.encode"
    };
    let (map, unmapped) = if let Some(path) = args.definition {
        let p =
            params::Substitution::parse(&definition(&path, op)?).map_err(|e| e.operation(op))?;
        (
            p.mapping().map_err(|e| e.operation(op))?,
            p.unmapped().to_owned(),
        )
    } else {
        let source = args.source.as_deref().ok_or_else(|| {
            OperationError::new(ErrorCode::InvalidParameter, "source alphabet is required")
                .operation(op)
                .parameter("source")
        })?;
        (
            alphabet::configured(source, args.target.as_deref(), args.keyword.as_deref())
                .map_err(|e| e.operation(op))?,
            args.unmapped,
        )
    };
    let text = args.input.read_text(op)?;
    Ok(if args.decode {
        map.decode(&text, &unmapped)?
    } else {
        map.encode(&text, &unmapped)?
    })
}
#[derive(Args)]
pub(crate) struct TableArgs {
    #[command(flatten)]
    input: Input,
    /// Exact local JSON table; <=64 KiB, never fetched or modified
    #[arg(long)]
    definition: PathBuf,
    #[arg(long)]
    decode: bool,
}
#[derive(Args)]
pub(crate) struct Homophonic {
    #[command(flatten)]
    input: Input,
    #[arg(long)]
    definition: PathBuf,
    /// Strict JSON array of canonical symbol/index records, 0..65535
    #[arg(long, default_value = "[]", conflicts_with = "decode")]
    counters: String,
    #[arg(long)]
    decode: bool,
}
fn definition(path: &std::path::Path, op: &'static str) -> Result<String, OperationError> {
    let file = cube_core::forensics::open_regular(path).map_err(|e| e.operation(op))?;
    let mut bytes = Vec::new();
    file.take(mono::MAX_DEFINITION as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| OperationError::io(e).operation(op).parameter("definition"))?;
    if bytes.len() > mono::MAX_DEFINITION {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "definition exceeds 64 KiB")
                .operation(op)
                .parameter("definition"),
        );
    }
    String::from_utf8(bytes).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "definition must be valid UTF-8")
            .operation(op)
            .parameter("definition")
    })
}
pub(crate) fn homophonic(args: Homophonic) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.homophonic.decode"
    } else {
        "cipher.homophonic.encode"
    };
    let map = homophonic::Mapping::parse(&definition(&args.definition, op)?)
        .map_err(|e| e.operation(op))?;
    let text = args.input.read_text(op)?;
    Ok(if args.decode {
        map.decode(&mono::symbol_tokens(&text)?)?
    } else {
        serde_json::to_string(&map.encode(&text, &mono::starting_counters(&args.counters)?)?)?
    })
}
pub(crate) fn digits(args: TableArgs, profile: Profile) -> Result<String, Box<dyn Error>> {
    let op = match (profile, args.decode) {
        (Profile::Straddling, false) => "cipher.straddling.encode",
        (Profile::Straddling, true) => "cipher.straddling.decode",
        (Profile::MonomeDinome, false) => "cipher.monome_dinome.encode",
        (Profile::MonomeDinome, true) => "cipher.monome_dinome.decode",
        (Profile::Tridigital, false) => "cipher.tridigital.encode",
        (Profile::Tridigital, true) => "cipher.tridigital.decode",
    };
    let table =
        Table::parse(profile, &definition(&args.definition, op)?).map_err(|e| e.operation(op))?;
    let text = args.input.read_text(op)?;
    Ok(if args.decode {
        if profile == Profile::Tridigital {
            serde_json::to_string(&table.alternatives(&text)?)?
        } else {
            table.decode(&text)?
        }
    } else {
        table.encode(&text)?
    })
}
pub(crate) fn grandpre(args: TableArgs) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.grandpre.decode"
    } else {
        "cipher.grandpre.encode"
    };
    let square =
        grandpre::Square::parse(&definition(&args.definition, op)?).map_err(|e| e.operation(op))?;
    let text = args.input.read_text(op)?;
    Ok(if args.decode {
        square.decode(&mono::coordinate_tokens(&text)?)?
    } else {
        serde_json::to_string(&square.encode(&text)?)?
    })
}
