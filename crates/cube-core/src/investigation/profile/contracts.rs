//! An explicit profile is required; metadata never silently chooses a profile.
use super::*;
use crate::operations::contracts::{self as c, Entry, Fixture, Port, schema, values};
use schemars::Schema;
use serde_json::json;
fn parameters() -> Schema {
    c::descriptor_parameters(&OPERATIONS[1])
}
fn fixture() -> std::result::Result<Fixture, crate::errors::OperationError> {
    let profile = Profile {
        identify: true,
        ..Profile::default()
    };
    let session = profile
        .run_bytes(b"ABC", None)
        .map_err(|e| *e.error)?
        .session;
    assert_eq!(session.identity().size, 3);
    assert_eq!(
        session.identity().digest.as_ref().unwrap().value,
        crate::hashing::sha256(b"ABC")
    );
    Ok(Fixture {
        inputs: json!({"profile":profile,"input":values::Bytes{data:b"ABC".to_vec()}}),
        parameters: json!({}),
        outputs: json!({"output":session}),
    })
}
pub(crate) const CONTRACTS: &[Entry] = &[Entry {
    operation_version: 2,
    descriptor: OPERATIONS[1],
    inputs: &[
        Port {
            name: "profile",
            semantic_type: "investigation.profile",
            schema: schema::<Profile>,
            required: true,
        },
        Port {
            name: "input",
            semantic_type: "byte_reader",
            schema: schema::<values::Bytes>,
            required: true,
        },
    ],
    parameters,
    outputs: &[Port {
        name: "output",
        semantic_type: "investigation.session",
        schema: schema::<Session>,
        required: true,
    }],
    limits: c::Limits {
        max_input_bytes: Some(session::MAX_INPUT_BYTES as u64),
        max_output_bytes: Some(session::MAX_RECORD_BYTES as u64),
        max_items: Some(64),
        max_work: None,
        max_memory_bytes: None,
    },
    fixture,
}];
