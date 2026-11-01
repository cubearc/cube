//! Positional transformations use the owning Core's explicit layouts.
use super::field;
use crate::{
    ciphers::{
        poly,
        transposition::{self as t, grille, routes},
    },
    errors::{ErrorCode, OperationError},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
type Handler = fn(&str, &Value, bool) -> Result<Value, OperationError>;
fn parse<T: DeserializeOwned>(p: &Value) -> Result<T, OperationError> {
    poly::parse_parameters(&p.to_string())
}
fn output(v: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(v).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "typed result serialization failed")
    })
}
macro_rules! text_handler {($name:ident,$f:path)=>{fn $name(t:&str,p:&Value,d:bool)->Result<Value,OperationError>{Ok(json!({"text":$f(t,&parse(p)?,d)?}))}};}
text_handler!(scytale, t::scytale);
text_handler!(rail_fence, t::rail_fence);
text_handler!(redefence, t::redefence);
text_handler!(sequence, t::sequence);
text_handler!(skip, t::skip);
text_handler!(columnar, t::columnar);
text_handler!(double_columnar, t::double_columnar);
text_handler!(myszkowski, t::myszkowski);
text_handler!(amsco, t::amsco);
text_handler!(route, routes::route);
text_handler!(spiral, routes::spiral);
text_handler!(caesar_box, routes::caesar_box);
text_handler!(turning_grille, grille::turning_grille);
fn complete(t: &str, p: &Value, d: bool) -> Result<Value, OperationError> {
    output(t::complete_columnar(t, &parse(p)?, d)?)
}
fn extract(t: &str, p: &Value, _: bool) -> Result<Value, OperationError> {
    Ok(json!({"text":grille::cardan_extract(t,&parse(p)?)?}))
}
fn overlay(t: &str, p: &Value, _: bool) -> Result<Value, OperationError> {
    output(grille::cardan_overlay(t, &parse(p)?)?)
}
fn resolve(op: &str) -> Option<Handler> {
    Some(match op {
        "cipher.scytale.encode" | "cipher.scytale.decode" => scytale,
        "cipher.rail_fence.encode" | "cipher.rail_fence.decode" => rail_fence,
        "cipher.redefence.encode" | "cipher.redefence.decode" => redefence,
        "cipher.sequence_transposition.encode" | "cipher.sequence_transposition.decode" => sequence,
        "cipher.skip.encode" | "cipher.skip.decode" => skip,
        "cipher.columnar.encode" | "cipher.columnar.decode" => columnar,
        "cipher.complete_columnar.encode" | "cipher.complete_columnar.decode" => complete,
        "cipher.double_columnar.encode" | "cipher.double_columnar.decode" => double_columnar,
        "cipher.myszkowski.encode" | "cipher.myszkowski.decode" => myszkowski,
        "cipher.amsco.encode" | "cipher.amsco.decode" => amsco,
        "cipher.route.encode" | "cipher.route.decode" => route,
        "cipher.spiral.encode" | "cipher.spiral.decode" => spiral,
        "cipher.caesar_box.encode" | "cipher.caesar_box.decode" => caesar_box,
        "cipher.turning_grille.encode" | "cipher.turning_grille.decode" => turning_grille,
        "cipher.cardan.extract" => extract,
        "cipher.cardan.overlay" => overlay,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    resolve(op).is_some()
}
pub(super) fn execute(
    op: &str,
    i: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let t: String = field(&i, "text")?;
    resolve(op).ok_or_else(|| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "no installed transposition binding",
        )
    })?(&t, &p, op.ends_with(".decode"))
}
