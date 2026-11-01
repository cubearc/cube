//! Explicit caller-defined mappings, with token boundaries preserved by Core.
use super::field;
use crate::{
    ciphers::{
        enigma,
        mono::{
            self,
            checkerboard::{Profile, Table},
            grandpre, homophonic, params,
        },
    },
    errors::{ErrorCode, OperationError},
};
use serde_json::{Value, json};
pub(super) fn supports(op: &str) -> bool {
    mono::OPERATIONS
        .iter()
        .chain(mono::historical::OPERATIONS)
        .any(|d| d.id == op)
        || op == "cipher.enigma"
}
pub(super) fn execute(
    op: &str,
    i: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    if mono::historical::OPERATIONS.iter().any(|d| d.id == op) {
        return mono::historical::execute(op, &i, &p);
    }
    let decode = op.ends_with(".decode");
    if op == "cipher.enigma" {
        return serde_json::to_value(enigma::transform(
            &field::<String>(&i, "text")?,
            &enigma::parse_parameters(&p.to_string())?,
        )?)
        .map_err(|_| invalid());
    }
    match op {
        "cipher.rot5" => Ok(json!({"text":mono::rotations::rot5(&field::<String>(&i,"text")?)?})),
        "cipher.rot18" => Ok(json!({"text":mono::rotations::rot18(&field::<String>(&i,"text")?)?})),
        "cipher.rot8000" => {
            Ok(json!({"text":mono::rotations::rot8000(&field::<String>(&i,"text")?)?}))
        }
        "cipher.rot47" => Ok(json!({"text":mono::rotations::rot47(&field::<String>(&i,"text")?)?})),
        "cipher.substitution.encode" | "cipher.substitution.decode" => {
            let params = params::Substitution::parse(&p.to_string())?;
            let mapping = params.mapping()?;
            let text: String = field(&i, "text")?;
            Ok(
                json!({"text":if decode {mapping.decode(&text,params.unmapped())?} else {mapping.encode(&text,params.unmapped())?}}),
            )
        }
        "cipher.homophonic.encode" | "cipher.homophonic.decode" => {
            let map = homophonic::Mapping::parse(&p["definition"].to_string())?;
            if decode {
                Ok(json!({"text":map.decode(&field::<Vec<String>>(&i,"tokens")?)?}))
            } else {
                Ok(
                    json!({"tokens":map.encode(&field::<String>(&i,"text")?,&field::<Vec<homophonic::Counter>>(&p,"counters")?)?}),
                )
            }
        }
        "cipher.grandpre.encode" | "cipher.grandpre.decode" => {
            let map = grandpre::Square::parse(&p["definition"].to_string())?;
            if decode {
                Ok(json!({"text":map.decode(&field::<Vec<grandpre::Token>>(&i,"tokens")?)?}))
            } else {
                Ok(json!({"tokens":map.encode(&field::<String>(&i,"text")?)?}))
            }
        }
        "cipher.straddling.encode"
        | "cipher.straddling.decode"
        | "cipher.monome_dinome.encode"
        | "cipher.monome_dinome.decode"
        | "cipher.tridigital.encode"
        | "cipher.tridigital.decode" => {
            let profile = if op.starts_with("cipher.straddling.") {
                Profile::Straddling
            } else if op.starts_with("cipher.monome_dinome.") {
                Profile::MonomeDinome
            } else {
                Profile::Tridigital
            };
            let table = Table::parse(profile, &p["definition"].to_string())?;
            let text: String = field(&i, "text")?;
            if decode && profile == Profile::Tridigital {
                Ok(json!({"alternatives":table.alternatives(&text)?}))
            } else {
                Ok(json!({"text":if decode {table.decode(&text)?}else{table.encode(&text)?}}))
            }
        }
        _ => Err(invalid()),
    }
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use an installed typed monoalphabetic profile",
    )
}
