use super::*;
use crate::operations::execution::{self as e, Entry, Failure, Representation::Text};
macro_rules! entry {
    ($id:literal,$function:path,$decode:literal)=>{Entry::pure($id,&[Text],&[Text],|i,p,_|{
        let parameters=crate::ciphers::poly::parse_parameters(&p.to_string()).map_err(|error|Failure(error.code.as_str()))?;
        $function(e::text(i,"text")?,&parameters,$decode).map(|text|serde_json::json!({"text":text})).map_err(|error|Failure(error.code.as_str()))
    })};
}
pub(crate) const ENTRIES: &[Entry] = &[
    entry!("cipher.cm_bifid.encode", cm::cm_bifid, false),
    entry!("cipher.cm_bifid.decode", cm::cm_bifid, true),
    entry!("cipher.vic.encode", vic::transform, false),
    entry!("cipher.vic.decode", vic::transform, true),
];
