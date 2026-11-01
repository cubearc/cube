//! CLI transports for Core-owned numerical puzzle profiles.
use cube_core::ciphers::{mono::numeric as n, poly};
use std::error::Error;
fn array(
    document: &str,
    op: &'static str,
) -> Result<Vec<String>, cube_core::errors::OperationError> {
    poly::parse_parameters(document).map_err(|mut e| {
        if e.code == cube_core::errors::ErrorCode::InvalidParameter {
            e.code = cube_core::errors::ErrorCode::InvalidInput;
        }
        e.operation(op).parameter("inputs")
    })
}
pub(crate) fn prime_number(args: crate::polycmd::Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.prime_number.decode"
    } else {
        "cipher.prime_number.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: n::prime::Parameters = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(if decode {
        serde_json::to_string(&n::prime::decode(&array(&text, op)?, &p)?)?
    } else {
        serde_json::to_string(&n::prime::encode(&text, &p)?)?
    })
}
pub(crate) fn prime_multiplication(
    args: crate::polycmd::Parameters,
) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.prime_multiplication.decode"
    } else {
        "cipher.prime_multiplication.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: n::product::Parameters =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(if decode {
        serde_json::to_string(&n::product::decode(&array(&text, op)?, &p)?)?
    } else {
        serde_json::to_string(&n::product::encode(&array(&text, op)?, &p)?)?
    })
}
pub(crate) fn modulo(args: crate::polycmd::Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.modulo.decode"
    } else {
        "cipher.modulo.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: n::modulo::Parameters =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(if decode {
        serde_json::to_string(&n::modulo::decode(&array(&text, op)?, &p)?)?
    } else {
        serde_json::to_string(&n::modulo::encode(&text, &p)?)?
    })
}
pub(crate) fn triliteral(args: crate::polycmd::Parameters) -> Result<String, Box<dyn Error>> {
    let op = if args.decode {
        "cipher.triliteral.decode"
    } else {
        "cipher.triliteral.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: n::triliteral::Parameters =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(if decode {
        serde_json::to_string(&n::triliteral::decode(&array(&text, op)?, &p)?)?
    } else {
        serde_json::to_string(&n::triliteral::encode(&text, &p)?)?
    })
}
