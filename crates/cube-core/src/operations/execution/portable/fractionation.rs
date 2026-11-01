//! Typed fractionation profiles call Core with explicit alphabets and periods.
use super::field;
use crate::{
    ciphers::{
        fractionation::{self as f, morse},
        poly,
    },
    errors::{ErrorCode, OperationError},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
type Handler = fn(&Value, &Value, bool) -> Result<Value, OperationError>;
fn parse<T: DeserializeOwned>(p: &Value) -> Result<T, OperationError> {
    poly::parse_parameters(&p.to_string())
}
fn output(v: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(v).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "typed result serialization failed")
    })
}
macro_rules! text_handler {($name:ident,$f:path)=>{fn $name(i:&Value,p:&Value,d:bool)->Result<Value,OperationError>{let t:String=field(i,"text")?;Ok(json!({"text":$f(&t,&parse(p)?,d)?}))}};}
macro_rules! report_handler {
    ($name:ident,$f:path) => {
        fn $name(i: &Value, p: &Value, d: bool) -> Result<Value, OperationError> {
            let t: String = field(i, "text")?;
            output($f(&t, &parse(p)?, d)?)
        }
    };
}
text_handler!(bifid, f::bifid);
text_handler!(trifid, f::trifid);
text_handler!(adfgx, f::adfgx);
text_handler!(adfgvx, f::adfgvx);
text_handler!(bazeries, f::bazeries);
report_handler!(fractionated_morse, morse::fractionated_morse);
report_handler!(morbit, morse::morbit);
report_handler!(pollux, morse::pollux);
fn nihilist(i: &Value, p: &Value, d: bool) -> Result<Value, OperationError> {
    if d {
        Ok(json!({"text":f::nihilist_decode(&field::<Vec<u16>>(i,"tokens")?,&parse(p)?)?}))
    } else {
        Ok(json!({"tokens":f::nihilist_encode(&field::<String>(i,"text")?,&parse(p)?)?}))
    }
}
fn resolve(op: &str) -> Option<Handler> {
    Some(match op {
        "cipher.bifid.encode" | "cipher.bifid.decode" => bifid,
        "cipher.trifid.encode" | "cipher.trifid.decode" => trifid,
        "cipher.adfgx.encode" | "cipher.adfgx.decode" => adfgx,
        "cipher.adfgvx.encode" | "cipher.adfgvx.decode" => adfgvx,
        "cipher.bazeries.encode" | "cipher.bazeries.decode" => bazeries,
        "cipher.fractionated_morse.encode" | "cipher.fractionated_morse.decode" => {
            fractionated_morse
        }
        "cipher.morbit.encode" | "cipher.morbit.decode" => morbit,
        "cipher.pollux.encode" | "cipher.pollux.decode" => pollux,
        "cipher.nihilist_substitution.encode" | "cipher.nihilist_substitution.decode" => nihilist,
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
    resolve(op).ok_or_else(|| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "no installed fractionation binding",
        )
    })?(&i, &p, op.ends_with(".decode"))
}
