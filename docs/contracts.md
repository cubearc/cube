# Operation contracts

Every implemented operation has a complete, opt-in contract for named inputs,
parameters, outputs and errors. Inspect it without executing anything:

```sh
cube ops describe image.histogram --json --contracts
cube ops list --json --contracts
```

```python
import cube
operation = cube.describe_operation("image.histogram", include_contracts=True)
assert operation["contract"]["outputs"][0]["semantic_type"] == "image.histogram"
```

Default discovery JSON, public `Descriptor` construction and existing direct
Rust functions retain their contracts. `--contracts`/`include_contracts=True`
adds canonical schemas; profile facts remain independently opt-in. No schema is
an execution dispatcher, plugin or filesystem/network permission.

## One source and versioned files

Rust descriptors and `schemars::JsonSchema` implementations on the actual domain
types are canonical. Module-owned entries add explicit ports and typed parameter
DTOs where legacy metadata alone cannot describe a direct signature. Custom
serialization annotations preserve Base64 bytes, hex and decimal counts. Unknown
semantic types fail rather than acquiring a name-only or permissive schema.

We chose generated versioned files instead of separately authored descriptor
JSON: typed Rust fields and serde serialization already own these structures.
This keeps them synchronized and avoids a duplicate manually maintained catalog.

```sh
cargo run --locked -p cube-core --example operation_contracts
cargo run --locked -p cube-core --example operation_contracts -- --check
```

The first command updates [the versioned catalog](../contracts/v1/catalog.json)
and [generated reference](reference/operations.md). The second rejects drift.
Owning-module registration and concrete-schema checks run in Cargo tests. A
fixture callback executes the real operation and checks an independent answer;
the contract gate then validates its named inputs, parameters and actual outputs.
Human-written guides explain tasks; generated references enumerate contracts.

## Values and safety

Schemas use JSON Schema 2020-12 and local references only. Validation uses the
maintained `jsonschema` library with HTTP/file resolvers disabled. Recursive
logical types use local references; actual parsers keep their depth/work bounds.
Schemas are trusted build-shipped data, never imported executable definitions.

Semantic type and transport representation are distinct. Bytes have explicit
Base64; rasters and PCM have typed metadata and exact sample bytes. Binary fields
omitted from legacy JSON reports become separate named data/image outputs.
Operations with multiple arguments expose named ports; generators can have none.
The current raster value follows Core's native 8-bit sample model. Core rejects
unsupported source depths; a browser preview never establishes sample precision.
Channel reports keep their bytes and optional image separate. Ordered pixel
exports describe traversal/corner/reversal, and comparison masks are optional
named raster outputs rather than silently omitted serde fields.
Compatible types still need actual format, dimension, capability and resource
validation by Core before execution. A null direct resource ceiling means that no
separate ceiling is declared there, not that the API is unbounded. Consult the
owning parser/API limits and retain interface budgets. When a descriptor argument
becomes a named input, its original bounds, choices and defaults stay on that port.
Canonical contracts are available only for exact registered descriptors.

Schema format and operation versions are independent. Legacy declarations start
at operation version 1; richer owning-module entries declare a positive
`operation_version`. Bump that version for an incompatible invocation/result
change and update the adapter, fixtures and generated reference together. This
keeps public `Descriptor` literals compatible; a package release does not silently
change every operation version.

Integers beyond JavaScript's exact range use canonical decimal strings on the
contract wire. `contracts::canonical_value` preserves their digits. The
`x-cube-integer` annotation retains original Rust bounds and decimal display
bounds; the `x-cube-native-integer` validator checks the string against them.
Ordinary safe integers remain JSON numbers. `x-cube-max-utf8-bytes` adds a byte
ceiling to scalar-count string bounds. Invalid values do not appear in errors.

Contract discovery has a four-MiB serialized ceiling, including its newline.
`max_output_bytes` can only lower it. Legacy discovery retains its one-MiB ceiling.
Conformance serialization has a separate 64-MiB streaming budget to accommodate
bounded Base64/JSON escaping; it does not enlarge an operation's raw input,
output, item, work, memory or interface limits. No partial discovery is returned.

A descriptor does not enable execution. Play must provide a bounded authorized
adapter and trusted viewer or useful structured fallback for every implemented
operation. Direct-only operations stay discoverable. Missing adapters, schemas,
fixtures or references remain delivery gaps. Secret capture is explicit and off
by default; private-export operations never become automatic recording actions.
