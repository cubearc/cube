//! Export/check the canonical catalog and references using standard Cargo tooling.
use cube_core::operations::contracts;
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let check = match std::env::args().nth(1).as_deref() {
        None => false,
        Some("--check") => true,
        _ => {
            return Err(
                "usage: cargo run -p cube-core --example operation_contracts -- [--check]".into(),
            );
        }
    };
    let document = contracts::document()?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("repository root missing")?;
    let mut catalog = serde_json::to_string_pretty(&document)?;
    catalog.push('\n');
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/operations/contracts/conformance.json"))?;
    let mut reference = String::from(
        "# Operation reference\n\nGenerated from Core descriptors and typed schemas. Edit their owning Rust declarations, then run `cargo run -p cube-core --example operation_contracts`. This reference lists implemented operations only.\n\nSchemas: [versioned catalog](../../contracts/v1/catalog.json). Structural contracts do not grant capabilities or replace operation runtime validation.\n\n",
    );
    for contract in &document.operations {
        use std::fmt::Write;
        writeln!(
            reference,
            "## {}\n\n{}\n\nCategory: `{}`. Contract: {}. Pipeline adapter: {}.\n",
            contract.operation,
            contract.description,
            contract.category,
            contract.operation_version,
            contract
                .pipeline_adapter
                .map_or_else(|| "none (direct API)".into(), |v| v.to_string())
        )?;
        writeln!(
            reference,
            "Play execution: query the authorized runtime's `/api/lab/v2/operations` availability and constraints. Direct API or pipeline support alone does not authorize a Play call.\n"
        )?;
        let fixture = fixtures
            .iter()
            .find(|f| f["operation"] == contract.operation)
            .ok_or("missing conformance example")?;
        contract.validate_fixture(&serde_json::from_value(fixture.clone())?)?;
        if fixture["operation_version"] != contract.operation_version
            || fixture["contract_sha256"]
                != cube_core::hashing::sha256(&serde_json::to_vec(contract)?)
        {
            return Err("review the changed contract and its conformance fixture".into());
        }
        writeln!(
            reference,
            "| Direction | Port | Semantic type |\n| --- | --- | --- |"
        )?;
        for (direction, ports) in [("Input", &contract.inputs), ("Output", &contract.outputs)] {
            for port in ports {
                writeln!(
                    reference,
                    "| {direction} | `{}` | `{}` |",
                    port.name, port.semantic_type
                )?;
            }
        }
        writeln!(
            reference,
            "\nFull named schemas, error/limit and privacy policy: [canonical catalog](../../contracts/v1/catalog.json). Real-result example: [synthetic conformance matrix](../../crates/cube-core/src/operations/contracts/conformance.json).\n"
        )?;
        if !contract.policy.private_input
            && !contract.policy.private_output
            && contract.policy.sensitive_parameters.is_empty()
        {
            let example =
                serde_json::json!({"inputs":fixture["inputs"],"parameters":fixture["parameters"]});
            let encoded = serde_json::to_string_pretty(&example)?;
            if encoded.len() <= 2048 {
                writeln!(
                    reference,
                    "Typed example (validated by the complete conformance matrix):\n\n```json\n{encoded}\n```\n"
                )?;
            }
        } else {
            writeln!(
                reference,
                "Private fixture values are omitted here; capture and retention require explicit interface policy.\n"
            )?;
        }
        writeln!(
            reference,
            "\nParameters:\n\n```json\n{}\n```\n\nExample:\n\n```sh\n{}\n```\n",
            serde_json::to_string_pretty(&contract.parameters)?,
            contract.example
        )?;
    }
    reference.truncate(reference.trim_end().len());
    reference.push('\n');
    for (path, content) in [
        (root.join("contracts/v1/catalog.json"), catalog),
        (root.join("docs/reference/operations.md"), reference),
    ] {
        if check {
            if std::fs::read_to_string(&path)? != content {
                return Err(format!(
                    "{} differs from canonical declarations; regenerate contracts",
                    path.display()
                )
                .into());
            }
        } else {
            std::fs::create_dir_all(path.parent().ok_or("output parent missing")?)?;
            std::fs::write(&path, content)?;
        }
    }
    println!(
        "{} complete operation contracts {}",
        document.operations.len(),
        if check { "verified" } else { "exported" }
    );
    Ok(())
}
