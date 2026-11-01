//! Typed classical-profile adapters. Core owns alphabet, key and padding rules.
use super::field;
use cube_core::{
    ciphers::poly::{self, geometry, hill, key_profiles, schedules, stateful, tableaux},
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
macro_rules! report_handler {
    ($name:ident,$f:path) => {
        fn $name(t: &str, p: &Value, d: bool) -> Result<Value, OperationError> {
            output($f(t, &parse(p)?, d)?)
        }
    };
}
text_handler!(gronsfeld, schedules::gronsfeld);
text_handler!(trithemius, schedules::trithemius);
text_handler!(autokey, schedules::autokey);
text_handler!(running_key, schedules::running_key);
text_handler!(interrupted_key, key_profiles::interrupted_key);
text_handler!(progressive_key, key_profiles::progressive_key);
text_handler!(multiplicative, key_profiles::multiplicative);
text_handler!(gromark, stateful::gromark);
text_handler!(periodic_gromark, stateful::periodic_gromark);
text_handler!(condi, stateful::condi);
text_handler!(ragbaby, stateful::ragbaby);
text_handler!(chaocipher, stateful::chaocipher);
text_handler!(solitaire, stateful::solitaire);
text_handler!(slidefair, tableaux::slidefair);
text_handler!(nicodemus, tableaux::nicodemus);
text_handler!(quagmire, tableaux::quagmire);
text_handler!(phillips, tableaux::phillips);
text_handler!(two_square, geometry::two_square);
text_handler!(four_square, geometry::four_square);
text_handler!(tri_square, geometry::tri_square);
report_handler!(playfair, geometry::playfair);
report_handler!(seriated_playfair, geometry::seriated_playfair);
report_handler!(hill, hill::hill);
report_handler!(digrafid, geometry::digrafid);
fn porta(t: &str, p: &Value, _: bool) -> Result<Value, OperationError> {
    Ok(json!({"text":schedules::porta(t,&parse(p)?)?}))
}
fn bellaso(t: &str, p: &Value, _: bool) -> Result<Value, OperationError> {
    Ok(json!({"text":poly::bellaso::transform(t,&parse(p)?)?}))
}
fn portax(t: &str, p: &Value, _: bool) -> Result<Value, OperationError> {
    Ok(json!({"text":tableaux::portax(t,&parse(p)?)?}))
}
fn resolve(op: &str) -> Option<Handler> {
    Some(match op {
        "cipher.gronsfeld.encode" | "cipher.gronsfeld.decode" => gronsfeld,
        "cipher.trithemius.encode" | "cipher.trithemius.decode" => trithemius,
        "cipher.autokey.encode" | "cipher.autokey.decode" => autokey,
        "cipher.running_key.encode" | "cipher.running_key.decode" => running_key,
        "cipher.interrupted_key.encode" | "cipher.interrupted_key.decode" => interrupted_key,
        "cipher.progressive_key.encode" | "cipher.progressive_key.decode" => progressive_key,
        "cipher.gromark.encode" | "cipher.gromark.decode" => gromark,
        "cipher.periodic_gromark.encode" | "cipher.periodic_gromark.decode" => periodic_gromark,
        "cipher.condi.encode" | "cipher.condi.decode" => condi,
        "cipher.ragbaby.encode" | "cipher.ragbaby.decode" => ragbaby,
        "cipher.chaocipher.encode" | "cipher.chaocipher.decode" => chaocipher,
        "cipher.solitaire.encode" | "cipher.solitaire.decode" => solitaire,
        "cipher.slidefair.encode" | "cipher.slidefair.decode" => slidefair,
        "cipher.nicodemus.encode" | "cipher.nicodemus.decode" => nicodemus,
        "cipher.quagmire.encode" | "cipher.quagmire.decode" => quagmire,
        "cipher.phillips.encode" | "cipher.phillips.decode" => phillips,
        "cipher.two_square.encode" | "cipher.two_square.decode" => two_square,
        "cipher.four_square.encode" | "cipher.four_square.decode" => four_square,
        "cipher.tri_square.encode" | "cipher.tri_square.decode" => tri_square,
        "cipher.playfair.encode" | "cipher.playfair.decode" => playfair,
        "cipher.seriated_playfair.encode" | "cipher.seriated_playfair.decode" => seriated_playfair,
        "cipher.hill.encode" | "cipher.hill.decode" => hill,
        "cipher.digrafid.encode" | "cipher.digrafid.decode" => digrafid,
        "cipher.vigenere_multiplicative.encode" | "cipher.vigenere_multiplicative.decode" => {
            multiplicative
        }
        "cipher.porta" => porta,
        "cipher.bellaso" => bellaso,
        "cipher.portax" => portax,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    resolve(op).is_some()
}
pub(super) fn execute(op: &str, i: Value, p: Value) -> Result<Value, OperationError> {
    let t: String = field(&i, "text")?;
    resolve(op).ok_or_else(|| {
        OperationError::new(ErrorCode::InvalidInput, "no installed polygraphic binding")
    })?(&t, &p, op.ends_with(".decode"))
}
