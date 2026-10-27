Describe the problem and resulting behavior. Link a related issue when relevant.

## Validation

- [ ] Add known-answer and edge-case tests for behavior changes.
- [ ] Run the relevant checks in CONTRIBUTING.md.
- [ ] Keep algorithms in cube-core; reuse them in the CLI and Python SDK.
- [ ] Update public API documentation and affected examples.

## Operation contracts

For every operation added or changed:

- [ ] Complete canonical input/parameter/output/error schemas, named ports and versions.
- [ ] Accurate bounds, sensitivity and capability requirements; no implicit uploads.
- [ ] Real positive/invalid/boundary conformance fixtures and compatible CLI/SDK behavior.
- [ ] Regenerated contract catalog/reference; the Cargo exporter `--check` passes.
- [ ] Required safe Play adapter and usable typed output handling are updated.
