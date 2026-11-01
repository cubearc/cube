//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use super::{field, images::bytes};
use crate::{
    blockchain::{self as key, bitcoin, candidates, ethereum, private, wif},
    errors::{ErrorCode, OperationError},
};
use serde_json::{Value, json};
enum Adapter {
    Bitcoin,
    Candidates,
    Ethereum,
    Export,
    Scalar,
    Wif,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "blockchain.bitcoin.derive" => Adapter::Bitcoin,
        "blockchain.candidates.validate" => Adapter::Candidates,
        "blockchain.ethereum.derive" => Adapter::Ethereum,
        "blockchain.private.export" => Adapter::Export,
        "blockchain.scalar.validate" => Adapter::Scalar,
        "blockchain.wif.validate" => Adapter::Wif,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use exact declared private material and explicit offline key options",
    )
}
fn text<'a>(p: &'a Value, k: &str) -> Result<&'a str, OperationError> {
    p[k].as_str().ok_or_else(invalid)
}
fn network(p: &Value, k: &str) -> Result<Option<key::Network>, OperationError> {
    p.get(k)
        .map(|_| key::Network::parse(text(p, k)?))
        .transpose()
}
pub(super) fn execute(
    op: &str,
    inputs: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let identity = crate::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| {
        if matches!(selected, Adapter::Wif) {
            return Ok(
                json!({"output":wif::validate(text(&inputs,"input")?,network(&p,"expected_network")?)?}),
            );
        }
        if matches!(selected, Adapter::Candidates) {
            let document = serde_json::to_vec(&inputs["input"]).map_err(|_| invalid())?;
            let entries = candidates::parse_json(&document)?;
            let derive = candidates::derivation(
                Some(text(&p, "derive")?),
                network(&p, "network")?,
                p.get("compressed")
                    .map(|_| field(&p, "compressed"))
                    .transpose()?,
                p.get("checksum")
                    .map(|_| text(&p, "checksum"))
                    .transpose()?,
            )?;
            let results: Vec<_> = candidates::CandidateBatch::new(entries, derive)?.collect();
            return Ok(json!({"output":results}));
        }
        let material = super::bytes(&inputs["input"])?;
        let input = key::PrivateInput::parse(text(&p, "format")?, &material)?;
        Ok(match selected {
            Adapter::Bitcoin => {
                json!({"output":bitcoin::derive(input,network(&p,"network")?.ok_or_else(invalid)?,field(&p,"compressed")?)?})
            }
            Adapter::Ethereum => {
                json!({"output":ethereum::derive(input,ethereum::Checksum::parse(text(&p,"checksum")?)?)?})
            }
            Adapter::Scalar => json!({"output":key::validate_scalar(input)?}),
            Adapter::Export => {
                json!({"output":bytes(&private::export(input,private::OutputFormat::parse(text(&p,"output")?)?,network(&p,"network")?,p.get("compressed").map(|_|field(&p,"compressed")).transpose()?)?.into_private_bytes())})
            }
            _ => return Err(invalid()),
        })
    })();
    result.map_err(|e: OperationError| e.operation(identity))
}
