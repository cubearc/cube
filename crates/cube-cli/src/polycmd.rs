//! Bounded file/text transport only. Every algorithm and validation lives in Core.
use crate::Input;
use clap::Args;
use cube_core::{
    ciphers::poly::{self, geometry, hill, schedules, stateful, tableaux},
    errors::{ErrorCode, OperationError},
};
use std::{error::Error, io::Read, path::PathBuf};
#[derive(Args)]
pub(crate) struct Parameters {
    #[command(flatten)]
    input: Input,
    /// Exact local typed JSON; <=2 MiB, no duplicate keys or unknown fields
    #[arg(long)]
    parameters: PathBuf,
    #[arg(long)]
    pub(crate) decode: bool,
}
pub(crate) fn transport(
    args: Parameters,
    op: &'static str,
) -> Result<(String, String, bool), Box<dyn Error>> {
    let file = cube_core::forensics::open_regular(&args.parameters).map_err(|e| e.operation(op))?;
    let mut bytes = Vec::new();
    file.take(poly::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| OperationError::io(e).operation(op).parameter("parameters"))?;
    if bytes.len() > poly::MAX_PARAMETERS {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "parameters exceed 2 MiB")
                .operation(op)
                .parameter("parameters")
                .into(),
        );
    }
    let document = String::from_utf8(bytes).map_err(|_| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "parameters require valid UTF-8",
        )
        .operation(op)
    })?;
    let input = args.input.read_bounded(4 * poly::MAX_INPUT, op)?;
    let text = String::from_utf8(input).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "text requires valid UTF-8").operation(op)
    })?;
    Ok((text, document, args.decode))
}
pub(crate) fn interrupted_key(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.interrupted_key.decode"
    } else {
        "cipher.interrupted_key.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: poly::key_profiles::InterruptedKey =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(poly::key_profiles::interrupted_key(&text, &p, decode)?)
}

pub(crate) fn progressive_key(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.progressive_key.decode"
    } else {
        "cipher.progressive_key.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: poly::key_profiles::ProgressiveKey =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(poly::key_profiles::progressive_key(&text, &p, decode)?)
}

pub(crate) fn vigenere_multiplicative(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.vigenere_multiplicative.decode"
    } else {
        "cipher.vigenere_multiplicative.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: poly::key_profiles::Multiplicative =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(poly::key_profiles::multiplicative(&text, &p, decode)?)
}
pub(crate) fn gronsfeld(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.gronsfeld.decode"
    } else {
        "cipher.gronsfeld.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: schedules::Gronsfeld = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(schedules::gronsfeld(&text, &p, decode)?)
}
pub(crate) fn trithemius(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.trithemius.decode"
    } else {
        "cipher.trithemius.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: schedules::Trithemius =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(schedules::trithemius(&text, &p, decode)?)
}
pub(crate) fn autokey(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.autokey.decode"
    } else {
        "cipher.autokey.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: schedules::Autokey = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(schedules::autokey(&text, &p, decode)?)
}
pub(crate) fn running_key(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.running_key.decode"
    } else {
        "cipher.running_key.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: schedules::RunningKey =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(schedules::running_key(&text, &p, decode)?)
}
pub(crate) fn bellaso(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = "cipher.bellaso";
    let (text, document, _) = transport(args, op)?;
    let p = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(poly::bellaso::transform(&text, &p)?)
}
pub(crate) fn porta(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = "cipher.porta";
    let (text, document, _decode) = transport(args, op)?;
    let p: schedules::Key = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(schedules::porta(&text, &p)?)
}
pub(crate) fn gromark(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.gromark.decode"
    } else {
        "cipher.gromark.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: stateful::Gromark = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(stateful::gromark(&text, &p, decode)?)
}
pub(crate) fn periodic_gromark(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.periodic_gromark.decode"
    } else {
        "cipher.periodic_gromark.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: stateful::PeriodicGromark =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(stateful::periodic_gromark(&text, &p, decode)?)
}
pub(crate) fn solitaire(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.solitaire.decode"
    } else {
        "cipher.solitaire.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: stateful::Solitaire = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(stateful::solitaire(&text, &p, decode)?)
}
pub(crate) fn slidefair(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.slidefair.decode"
    } else {
        "cipher.slidefair.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: schedules::Key = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(tableaux::slidefair(&text, &p, decode)?)
}
pub(crate) fn portax(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = "cipher.portax";
    let (text, document, _decode) = transport(args, op)?;
    let p: schedules::Key = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(tableaux::portax(&text, &p)?)
}
pub(crate) fn ragbaby(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.ragbaby.decode"
    } else {
        "cipher.ragbaby.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: stateful::Ragbaby = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(stateful::ragbaby(&text, &p, decode)?)
}
pub(crate) fn condi(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.condi.decode"
    } else {
        "cipher.condi.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: stateful::Condi = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(stateful::condi(&text, &p, decode)?)
}
pub(crate) fn nicodemus(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.nicodemus.decode"
    } else {
        "cipher.nicodemus.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: tableaux::Nicodemus = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(tableaux::nicodemus(&text, &p, decode)?)
}
pub(crate) fn quagmire(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.quagmire.decode"
    } else {
        "cipher.quagmire.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: tableaux::Quagmire = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(tableaux::quagmire(&text, &p, decode)?)
}
pub(crate) fn phillips(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.phillips.decode"
    } else {
        "cipher.phillips.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: tableaux::Phillips = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(tableaux::phillips(&text, &p, decode)?)
}
pub(crate) fn chaocipher(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.chaocipher.decode"
    } else {
        "cipher.chaocipher.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: stateful::Chaocipher = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(stateful::chaocipher(&text, &p, decode)?)
}
pub(crate) fn playfair(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.playfair.decode"
    } else {
        "cipher.playfair.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: geometry::Playfair = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&geometry::playfair(
        &text, &p, decode,
    )?)?)
}
pub(crate) fn seriated_playfair(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.seriated_playfair.decode"
    } else {
        "cipher.seriated_playfair.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: geometry::SeriatedPlayfair =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&geometry::seriated_playfair(
        &text, &p, decode,
    )?)?)
}
pub(crate) fn two_square(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.two_square.decode"
    } else {
        "cipher.two_square.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: geometry::Squares = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(geometry::two_square(&text, &p, decode)?)
}
pub(crate) fn four_square(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.four_square.decode"
    } else {
        "cipher.four_square.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: geometry::Squares = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(geometry::four_square(&text, &p, decode)?)
}
pub(crate) fn hill(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.hill.decode"
    } else {
        "cipher.hill.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: hill::Hill = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&hill::hill(&text, &p, decode)?)?)
}
pub(crate) fn digrafid(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.digrafid.decode"
    } else {
        "cipher.digrafid.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: geometry::Digrafid = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&geometry::digrafid(
        &text, &p, decode,
    )?)?)
}
pub(crate) fn tri_square(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.tri_square.decode"
    } else {
        "cipher.tri_square.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: geometry::TriSquare = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(geometry::tri_square(&text, &p, decode)?)
}
