# Architecture

Cube is a Cargo workspace with one Rust implementation shared by its interfaces.
User workflows belong in the [CLI](cli.md), [Python](python.md) and
[Operations](operations.md) guides. This document describes contributor-facing
boundaries and extension contracts.

## Responsibilities

| Package | Responsibility |
| --- | --- |
| `crates/cube-core` | Reusable operations, typed results, validation and errors |
| `crates/cube-cli` | Clap arguments, caller-selected I/O, display and exit codes |
| `bindings/python` | Python types/exceptions, exports, stubs and native packaging |

The root `pyproject.toml` builds the PyO3 extension with Maturin. Source
distributions retain the workspace members, shared Cargo.lock, README and licenses.
SDKs call Rust functions; they do not duplicate algorithms or use CLI subprocesses.

## Core functions and data

Operations remain ordinary public Rust functions or cohesive concrete types.
Binary transformations use bytes; encodings/digests return specified text; classical
ciphers accept UTF-8 and preserve characters outside their documented alphabets.
Direct APIs do not adopt a universal runtime value or runtime-dispatched trait.

Maintained crates supply encoding, hash and format primitives. Small original
transformations stay in their owning modules. Responsibilities include:

- `encoding`, `ciphers`, `hashing`, `binary`, `text`, `symbols`, `numbers`: explicit
  transformations. Unicode normalization uses pinned data, exposed through
  `text::UNICODE_VERSION`; dependency/data changes require compatibility review.
- `forensics`: checked ranges, signature evidence, file facts, bounded EXIF, byte
  inspection and bounded PNG/JPEG/GIF framing. Reader/writer APIs support bounded or streaming work
  without requiring paths. Signatures are candidates, not validity proofs.
- `image`: an immutable validated PNG8 sample model, with separate analysis,
  geometry/color and bit-extraction modules. Indexed samples, palette and tRNS
  remain separate until explicit expansion. PNG framing reuses `forensics`.
- `media`: WAVE framing, immutable selected PCM blocks and incremental min/max
  envelopes. Metadata skips payloads with `Read + Seek`; extraction/envelopes share
  frame-aligned bounded streaming. Block format/channel/continuity checks precede
  consumption. No native multimedia or graphical runtime is required.

Resource bounds and unsupported formats belong beside the implementation; the
[Operations Guide](operations.md) summarizes those affecting users.

`forensics::boundaries` dispatches supported image framing and reuses strict PNG
inspection. JPEG/GIF parsers use bounded buffered readers, checked extents and a
record cap; they do not allocate decoded images. The existing PNG boundary struct
and descriptor type name are retained for compatibility. Rules follow the
[GIF89a specification](https://www.w3.org/Graphics/GIF/spec-gif89a.txt) and the
supported baseline subset of [ITU T.81](https://www.w3.org/Graphics/JPEG/itu-t81.pdf).

`forensics::zip` checks a documented classic single-disk structural subset from
[PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT).
A bounded prefix and cumulative candidate/header/metadata budgets prevent repeated
EOCD decoys from causing unbounded parsing. Local extents are checked against the
central directory and sorted for overlap/gap rejection. `boundaries` reuses the
validator; no archive decompressor, entry execution or second ZIP parser is added.

`forensics::carving` validates an explicit origin through a checked relative
reader, then reuses `copy_range` for exclusive output. A small writer adapter
computes the recovered content's SHA-256 while bytes stream. Provenance contains
ranges/evidence/digest without automatically persisting source paths or content in
sessions. Filesystem I/O stays outside recipes; caller-selected raw ranges never
acquire a format-validation claim. Metadata changes trigger new-output cleanup;
source stability remains a caller responsibility rather than a snapshot guarantee.

## Metadata and discovery

Static descriptors live beside operations and are assembled by
`operations::catalog`. Stable operation identifiers, semantic input/output types,
parameter constraints/defaults, descriptions, examples, determinism and reversibility
form the discovery contract. Categories are strings, so adding a category does not
require restructuring a central enum.

The CLI and SDK reuse this catalog. It describes operations without executing
them, and it is not a parallel dispatch hierarchy. Add new fields/identifiers
compatibly; never reuse an identifier for different behavior. Preserve public
functions and existing binary/text semantics. CLI help, Rustdoc and Python stubs
remain the argument/API references; Markdown guides explain workflows.

`Descriptor::discovery()` adds pipeline availability by querying the executor's
actual allowlist. Its flattened JSON view preserves descriptor fields and adds
`pipeline: {"version": 1}` for an adapter or `pipeline: null` otherwise; the public
descriptor layout and ordinary catalog APIs remain unchanged. There is no second
adapter registry. Tests audit unique/known typed adapter references, catalog CLI
routes, and native Python exports/stubs, including class methods.

## Interface and error boundaries

The CLI reads exact bytes unless an operation explicitly needs UTF-8. It controls
stdin/stdout, new destination files and presentation. File inspection opens regular
files read-only; Cube destination APIs use exclusive creation. Raw binary output
adds neither a newline nor metadata.

Python functions translate values and call the core. Binary results use `bytes`;
structured metadata uses dictionaries/lists, sometimes containing native bytes or
image objects. Type stubs describe the public API. Input validation uses typed
core errors; legacy variants/public signatures remain compatible. `OperationError`
adds stable category/operation/parameter/offset context and retains original causes.
Context identifies fields and offsets rather than dumping full inputs or paths.

## Pipelines and recipes

Linear pipelines are implemented in `pipeline`. Small explicit, versioned adapters
reuse direct functions, descriptors and validators. Only supported bounded adapters
execute; discovery alone does not grant execution. Immutable validated plans check
IDs, versions, parameters and adjacent types. Execution checks actual input and
conservative value/work/capture bounds before transformations, stops at the first
failure and supports cooperative cancellation at bounded operation boundaries.
Pipeline-only values distinguish bytes, text and explicitly supported structured
kinds. Future adapters must declare concrete types and resource bounds; direct
function APIs remain independent of this composition boundary.

The engine owns strict JSON recipe parsing, validation, compatibility and execution.
A shared bounded unique-key parser rejects ambiguous documents. Format/engine and
adapter versions are explicit contracts. Recipe input slots hold caller-supplied
values and confer no I/O capability. Community definitions/examples belong in
`cubearc/recipes`, with independent compatibility checks and no copied algorithms.

The `forensic_report` pipeline value is a closed enum of owned signature, hex and
printable-string snapshots. It exists to preserve structured evidence without
turning it into UTF-8 text or an arbitrary JSON value. Reader-based inspections
use an explicit bytes-to-report adapter contract over `Cursor`; direct descriptors
and APIs retain their original types. Adapter listings declare these kinds.
Conservative output/work/capture estimates run before algorithms. Existing values,
recipes and adapter versions retain their behavior; exhaustive Rust matches on
`Kind`/`Value` must handle the new report variant. Report deserialization validates
budgets and internal range/data consistency; caller-supplied evidence is still data.

## Investigations

`investigation` reuses signature evidence, the available catalog and existing
operations/pipelines. Explicit rules produce explainable suggestions. Sessions
separate machine observations/candidates from linked user hypotheses, preserve
session-local input/run/finding IDs, and bound opt-in result/parameter retention.
Session model, execution and record/replay modules keep those responsibilities apart.
Strict loading validates versions, references and states; replay rejects missing
identity or redaction gaps before operations execute.

Versioned profiles preflight a concrete metadata/preview allowlist and optional
pipeline. One immutable bounded prefix feeds shared session execution; optional
full-input identity is an explicit charged run. Safe profile context/counters
are recorded, while elapsed time remains diagnostic. JSON export selects content
explicitly; human reports escape untrusted strings. There is no implicit saving,
storage backend, telemetry or network capability.

## Extend and verify

Follow [Contributing](../CONTRIBUTING.md#add-an-operation) for focused changes and
[quality checks](../CONTRIBUTING.md#quality-checks). Tests combine known answers,
binary round trips, invalid/boundary inputs, CLI integration and the real Python
extension. CI enforces the core coverage threshold independently of external uploads.

Additional SDKs should translate their language's idioms onto the Rust core.
Introduce shared FFI machinery only when an implemented binding demonstrates a
concrete need. A plugin runtime, distributed workflows and visual workbench are
not implemented; future privileged extensions need a separate compatibility,
permission, sandboxing and resource-limit design.
