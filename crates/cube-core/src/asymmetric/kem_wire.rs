//! Shared byte-port declarations for raw key encapsulation contracts.
use crate::operations::contracts as c;
use serde_json::json;

pub(super) fn byte_schema<const MIN: usize, const MAX: usize>() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    let property = &mut schema["properties"]["base64"];
    property["minLength"] = json!(MIN.div_ceil(3) * 4);
    property["maxLength"] = json!(MAX.div_ceil(3) * 4);
    property["x-cube-min-decoded-bytes"] = json!(MIN);
    property["x-cube-max-decoded-bytes"] = json!(MAX);
    schemars::Schema::try_from(schema).expect("bounded KEM byte port")
}

pub(super) const fn byte_port(name: &'static str, schema: fn() -> schemars::Schema) -> c::Port {
    c::Port {
        name,
        semantic_type: "bytes",
        schema,
        required: true,
    }
}
