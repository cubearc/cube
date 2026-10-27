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
