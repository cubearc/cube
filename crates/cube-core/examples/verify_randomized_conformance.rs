//! Qualification-only adapter for a fixed reviewed randomized fixture.
//! Not installed with Cube or exposed through operation execution APIs.
#[path = "../tests/support/randomized_fixture.rs"]
mod randomized_fixture;
use std::io::Read;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let operation = std::env::args()
        .nth(1)
        .ok_or("provide a reviewed fixture operation")?;
    let mut input = Vec::new();
    std::io::stdin()
        .take(1024 * 1024 + 1)
        .read_to_end(&mut input)?;
    if input.len() > 1024 * 1024 {
        return Err("conformance output exceeds its test-tool bound".into());
    }
    let actual: serde_json::Value =
        serde_json::from_slice(&input).map_err(|_| "invalid conformance output")?;
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/operations/contracts/conformance.json"))?;
    let case = cases
        .iter()
        .find(|v| v["operation"] == operation)
        .ok_or("reviewed fixture missing")?;
    let contract = cube_core::operations::require(&operation)?.contract()?;
    contract
        .validate_outputs(&actual)
        .map_err(|_| "conformance output schema mismatch")?;
    std::panic::set_hook(Box::new(|_| {}));
    std::panic::catch_unwind(|| randomized_fixture::assert_inverse(case, &actual))
        .map_err(|_| "conformance inverse verification failed")?;
    println!(
        "{}",
        serde_json::json!({"operation":operation,"comparison":"verified-inverse","verified":true})
    );
    Ok(())
}
