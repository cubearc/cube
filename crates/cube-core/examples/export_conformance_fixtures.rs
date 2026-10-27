//! Export fixed synthetic owning-module fixtures for cross-interface conformance.
use cube_core::operations::{self, contracts};
use serde_json::json;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("supply an existing fixture directory")?,
    );
    for (operation, fixture) in contracts::registered_fixtures()? {
        let c = operations::require(&operation)?.contract()?;
        let value = contracts::canonical_value(
            json!({"operation":operation,"operation_version":c.operation_version,"contract_sha256":cube_core::hashing::sha256(&serde_json::to_vec(&c)?),"inputs":fixture.inputs,"parameters":fixture.parameters,"outputs":fixture.outputs}),
        );
        std::fs::write(
            directory.join(format!("{operation}.json")),
            serde_json::to_vec(&value)?,
        )?;
    }
    Ok(())
}
