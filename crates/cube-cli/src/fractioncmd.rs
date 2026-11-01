//! Bounded local transport only; all algorithms and validation are in Core.
use crate::polycmd::{Parameters, transport};
use cube_core::ciphers::{fractionation, poly, transposition};
use std::error::Error;
pub(crate) fn scytale(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.scytale.decode"
    } else {
        "cipher.scytale.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Scytale =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::scytale(&text, &p, decode)?)
}
pub(crate) fn rail_fence(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.rail_fence.decode"
    } else {
        "cipher.rail_fence.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Rails = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::rail_fence(&text, &p, decode)?)
}
pub(crate) fn redefence(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.redefence.decode"
    } else {
        "cipher.redefence.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Redefence =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::redefence(&text, &p, decode)?)
}
pub(crate) fn sequence_transposition(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.sequence_transposition.decode"
    } else {
        "cipher.sequence_transposition.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Sequence =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::sequence(&text, &p, decode)?)
}
pub(crate) fn skip(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.skip.decode"
    } else {
        "cipher.skip.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Skip = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::skip(&text, &p, decode)?)
}
pub(crate) fn columnar(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.columnar.decode"
    } else {
        "cipher.columnar.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Columnar =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::columnar(&text, &p, decode)?)
}
pub(crate) fn complete_columnar(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.complete_columnar.decode"
    } else {
        "cipher.complete_columnar.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::CompleteColumnar =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&transposition::complete_columnar(
        &text, &p, decode,
    )?)?)
}
pub(crate) fn double_columnar(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.double_columnar.decode"
    } else {
        "cipher.double_columnar.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::DoubleColumnar =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::double_columnar(&text, &p, decode)?)
}
pub(crate) fn myszkowski(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.myszkowski.decode"
    } else {
        "cipher.myszkowski.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Myszkowski =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::myszkowski(&text, &p, decode)?)
}
pub(crate) fn amsco(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.amsco.decode"
    } else {
        "cipher.amsco.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::Amsco = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::amsco(&text, &p, decode)?)
}
pub(crate) fn route(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.route.decode"
    } else {
        "cipher.route.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::routes::Routes =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::routes::route(&text, &p, decode)?)
}
pub(crate) fn spiral(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.spiral.decode"
    } else {
        "cipher.spiral.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::routes::Routes =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::routes::spiral(&text, &p, decode)?)
}
pub(crate) fn caesar_box(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.caesar_box.decode"
    } else {
        "cipher.caesar_box.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::routes::CaesarBox =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::routes::caesar_box(&text, &p, decode)?)
}
pub(crate) fn turning_grille(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.turning_grille.decode"
    } else {
        "cipher.turning_grille.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::grille::TurningGrille =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(transposition::grille::turning_grille(&text, &p, decode)?)
}
pub(crate) fn bifid(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.bifid.decode"
    } else {
        "cipher.bifid.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::Bifid = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(fractionation::bifid(&text, &p, decode)?)
}
pub(crate) fn trifid(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.trifid.decode"
    } else {
        "cipher.trifid.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::Trifid =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(fractionation::trifid(&text, &p, decode)?)
}
pub(crate) fn adfgx(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.adfgx.decode"
    } else {
        "cipher.adfgx.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::Adfgx = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(fractionation::adfgx(&text, &p, decode)?)
}
pub(crate) fn adfgvx(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.adfgvx.decode"
    } else {
        "cipher.adfgvx.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::Adfgvx =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(fractionation::adfgvx(&text, &p, decode)?)
}
pub(crate) fn nihilist_substitution(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.nihilist_substitution.decode"
    } else {
        "cipher.nihilist_substitution.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::Nihilist =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    if decode {
        let tokens = fractionation::parse_nihilist_tokens(&text)?;
        Ok(fractionation::nihilist_decode(&tokens, &p)?)
    } else {
        Ok(serde_json::to_string(&fractionation::nihilist_encode(
            &text, &p,
        )?)?)
    }
}
pub(crate) fn bazeries(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.bazeries.decode"
    } else {
        "cipher.bazeries.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::Bazeries =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(fractionation::bazeries(&text, &p, decode)?)
}
pub(crate) fn fractionated_morse(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.fractionated_morse.decode"
    } else {
        "cipher.fractionated_morse.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::morse::FractionatedMorse =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(
        &fractionation::morse::fractionated_morse(&text, &p, decode)?,
    )?)
}
pub(crate) fn morbit(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.morbit.decode"
    } else {
        "cipher.morbit.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::morse::Morbit =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&fractionation::morse::morbit(
        &text, &p, decode,
    )?)?)
}
pub(crate) fn pollux(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.pollux.decode"
    } else {
        "cipher.pollux.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::morse::Pollux =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&fractionation::morse::pollux(
        &text, &p, decode,
    )?)?)
}
pub(crate) fn cardan_extract(args: Parameters) -> Result<String, Box<dyn Error>> {
    let (text, document, decode) = transport(args, "cipher.cardan.extract")?;
    if decode {
        return Err("Cardan profiles have no inverse flag".into());
    }
    let p: transposition::grille::Cardan = poly::parse_parameters(&document)?;
    Ok(transposition::grille::cardan_extract(&text, &p)?)
}
pub(crate) fn cardan_overlay(args: Parameters) -> Result<String, Box<dyn Error>> {
    let (text, document, decode) = transport(args, "cipher.cardan.overlay")?;
    if decode {
        return Err("Cardan profiles have no inverse flag".into());
    }
    let p: transposition::grille::Overlay = poly::parse_parameters(&document)?;
    Ok(serde_json::to_string(
        &transposition::grille::cardan_overlay(&text, &p)?,
    )?)
}

pub(crate) fn cadenus(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.cadenus.decode"
    } else {
        "cipher.cadenus.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::tables::Cadenus =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let result = transposition::tables::cadenus(&text, &p, decode)?;
    Ok(serde_json::to_string(&result)?)
}

pub(crate) fn swagman(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.swagman.decode"
    } else {
        "cipher.swagman.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::tables::Swagman =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let result = transposition::tables::swagman(&text, &p, decode)?;
    Ok(serde_json::to_string(&result)?)
}

pub(crate) fn nihilist_transposition(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.nihilist_transposition.decode"
    } else {
        "cipher.nihilist_transposition.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::tables::Nihilist =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let result = transposition::tables::nihilist(&text, &p, decode)?;
    Ok(serde_json::to_string(&result)?)
}

pub(crate) fn ubchi(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.ubchi.decode"
    } else {
        "cipher.ubchi.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: transposition::ubchi::Ubchi =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let result = transposition::ubchi::transform(&text, &p, decode)?;
    Ok(serde_json::to_string(&result)?)
}

pub(crate) fn cm_bifid(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.cm_bifid.decode"
    } else {
        "cipher.cm_bifid.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::cm::CmBifid =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let result = fractionation::cm::cm_bifid(&text, &p, decode)?;
    Ok(result)
}

pub(crate) fn vic(args: Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.vic.decode"
    } else {
        "cipher.vic.encode"
    };
    let (text, document, decode) = transport(args, op)?;
    let p: fractionation::vic::VicEnglish2010 =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let result = fractionation::vic::transform(&text, &p, decode)?;
    Ok(result)
}
