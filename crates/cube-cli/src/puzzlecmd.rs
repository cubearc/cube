//! Thin adapters for explicit puzzle alphabets and mappings.
use crate::Input;
use clap::Args;
use cube_core::{
    ciphers::{bacon, polybius},
    symbols::keypad,
    text::{classes, keyboard},
};
use std::{error::Error, io::Write};
#[derive(Args)]
pub(crate) struct BaconArgs {
    #[command(flatten)]
    input: Input,
    /// classical24 (J=I,V=U) or extended26
    #[arg(long)]
    variant: String,
    /// ab, binary (A=0) or binary_inverted (A=1)
    #[arg(long, default_value = "ab")]
    notation: String,
    #[arg(long)]
    decode: bool,
}
pub(crate) fn bacon(args: BaconArgs) -> Result<String, Box<dyn Error>> {
    let input = args.input.read_text(if args.decode {
        "cipher.bacon.decode"
    } else {
        "cipher.bacon.encode"
    })?;
    Ok(if args.decode {
        bacon::decode(&input, &args.variant, &args.notation)?
    } else {
        bacon::encode(&input, &args.variant, &args.notation)?
    })
}
#[derive(Args)]
pub(crate) struct PolybiusArgs {
    #[command(flatten)]
    input: Input,
    /// polybius (I/J) or tap (C/K); conflicts with custom alphabet
    #[arg(long, conflicts_with = "alphabet")]
    preset: Option<String>,
    /// Unique exact Unicode square; requires --side; spaces reserved for word tokens
    #[arg(long, requires = "side")]
    alphabet: Option<String>,
    #[arg(long, requires = "alphabet")]
    side: Option<u8>,
    /// ALIAS=CANONICAL, exactly one scalar each; custom square only
    #[arg(long = "alias", requires = "alphabet")]
    aliases: Vec<String>,
    #[arg(long, default_value_t = 1)]
    index_base: u8,
    /// Decode exact ROW,COLUMN or / tokens, separated by one ASCII space
    #[arg(long)]
    decode: bool,
    /// Encode structured coordinate arrays with null for each literal space
    #[arg(long)]
    json: bool,
}
pub(crate) fn polybius(args: PolybiusArgs) -> Result<String, Box<dyn Error>> {
    if args.aliases.len() > 4096 {
        return Err(cube_core::errors::OperationError::new(
            cube_core::errors::ErrorCode::ResourceLimit,
            "square aliases exceed 4096 entries",
        )
        .parameter("aliases")
        .into());
    }
    let square = if let Some(alphabet) = args.alphabet {
        let aliases = args
            .aliases
            .iter()
            .map(|a| {
                let mut scalars = a.chars();
                match (
                    scalars.next(),
                    scalars.next(),
                    scalars.next(),
                    scalars.next(),
                ) {
                    (Some(alias), Some('='), Some(target), None) => Ok((alias, target)),
                    _ => Err(cube_core::errors::OperationError::new(
                        cube_core::errors::ErrorCode::InvalidParameter,
                        "alias must be SCALAR=SCALAR",
                    )
                    .parameter("aliases")),
                }
            })
            .collect::<Result<Vec<_>, cube_core::errors::OperationError>>()?;
        polybius::Square::new(args.side.expect("clap requires side"), &alphabet, &aliases)?
    } else {
        polybius::Square::preset(args.preset.as_deref().unwrap_or("polybius"))?
    };
    let text = args.input.read_text(if args.decode {
        "cipher.polybius.decode"
    } else {
        "cipher.polybius.encode"
    })?;
    if args.decode {
        Ok(square.decode(&polybius::parse_tokens(&text)?, args.index_base)?)
    } else {
        let tokens = square.encode(&text, args.index_base)?;
        Ok(if args.json {
            serde_json::to_string(&tokens)?
        } else {
            polybius::format_tokens(&tokens)?
        })
    }
}
#[derive(Args)]
pub(crate) struct KeypadArgs {
    #[command(flatten)]
    input: Input,
    /// telephone (lossy digit alternatives) or multi_tap (delimited groups)
    #[arg(long)]
    mode: String,
    /// Strict JSON: {"ascii_fold":false,"keys":[{"digit":"0","symbols":" "}]}
    #[arg(long)]
    mapping: Option<String>,
}
pub(crate) fn keypad(args: KeypadArgs, decode: bool) -> Result<String, Box<dyn Error>> {
    let mapping = args
        .mapping
        .as_deref()
        .map(keypad::Mapping::parse)
        .transpose()?
        .unwrap_or_else(keypad::Mapping::standard);
    let text = args.input.read_text(if decode {
        "symbol.keypad.decode"
    } else {
        "symbol.keypad.encode"
    })?;
    if decode {
        Ok(serde_json::to_string(&mapping.decode(&text, &args.mode)?)?)
    } else {
        Ok(mapping.encode(&text, &args.mode)?)
    }
}
#[derive(Args)]
pub(crate) struct KeyboardArgs {
    #[command(flatten)]
    input: Input,
    /// qwerty, qwertz (Y/Z swap only), dvorak or strict JSON Layout
    #[arg(long)]
    source: String,
    #[arg(long)]
    target: String,
    #[arg(long, default_value = "preserve")]
    unmapped: String,
}
pub(crate) fn keyboard(args: KeyboardArgs) -> Result<String, Box<dyn Error>> {
    Ok(keyboard::remap(
        &args.input.read_text("text.keyboard.remap")?,
        &keyboard::Layout::parse(&args.source)?,
        &keyboard::Layout::parse(&args.target)?,
        &args.unmapped,
    )?)
}
#[derive(Args)]
pub(crate) struct ClassArgs {
    #[command(flatten)]
    input: Input,
    /// Required when preset input contains Y: vowel, consonant or unmapped
    #[arg(long, conflicts_with = "definition")]
    y: Option<String>,
    /// Strict JSON with ascii_fold and disjoint classes (name, characters, symbol)
    #[arg(long)]
    definition: Option<String>,
    #[arg(long, default_value = "error")]
    unmapped: String,
}
pub(crate) fn classes(args: ClassArgs, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let text = args.input.read_text("text.class.map")?;
    let report = if let Some(definition) = args.definition {
        classes::Definition::parse(&definition)?.map(&text, &args.unmapped)?
    } else {
        classes::map(&text, args.y.as_deref(), &args.unmapped)?
    };
    writeln!(output, "{}", serde_json::to_string(&report)?)?;
    Ok(())
}
