# Profile discovery

Cube can describe the implemented cryptographic and puzzle profiles without
executing them. Request the optional `metadata` object explicitly:

```sh
cube ops list --category ciphers --json --metadata
cube ops describe hash.sha256 --json --metadata
cube ops describe cipher.vigenere.decode
```

```python
import cube

sha = cube.describe_operation("hash.sha256", include_metadata=True)
assert sha["metadata"]["canonical_name"] == "SHA-256"
assert sha["metadata"]["profile"]["output_bits"] == 256
assert sha["pipeline"] == {"version": 1}
assert cube.sha256(b"abc") == (
    "ba7816bf8f01cfea414140de5dae2223"
    "b00361a396177a9cb410ff61f20015ad"
)
```

In Rust, use `Descriptor::metadata()` for the typed optional facts,
`Descriptor::profiled_discovery()` for their flattened serializable view, or
`operations::discovery_document` for a bounded document. Original `Descriptor`
and `Discovery` literal construction and their serialization remain unchanged.
CLI JSON without `--metadata` and Python without `include_metadata=True` retain
all previous fields and values. The CLI envelope remains `schema_version: 1`;
the optional object has its own `schema_version: 1`. Human descriptions append
profile facts. Human listings retain the existing command order.

## Measured gaps and their representation

The existing descriptor already provides stable ID, category, description,
semantic input/output type names, parameter constraints/defaults, determinism,
reversibility, CLI command and example. `Discovery.pipeline` already reports the
actual bounded adapter version. None of those fields needs a duplicate.

| Gap in the original view | Optional representation | Why it is separate |
| --- | --- | --- |
| No annotation version | `metadata.schema_version` | Versions these facts independently of package, recipe and adapter contracts. |
| Display names and grouping implicit in IDs/prose | `canonical_name`, `family` | Names the exact implemented profile. |
| Qualified reuse hidden in prose | `aliases[].name`, `aliases[].usage` | Records parameter or direction qualifications without registering new IDs. |
| Purpose and security limitations mixed with descriptions | `security.classification`, `security.limitations` | Distinguishes historical ciphers, encodings, unkeyed/legacy digests, checksums, transforms and private/public material. |
| Intrinsic private material not identified | `security.private_input`, `private_output`, `sensitive_parameters` | Describes known sensitivity without including values. Unflagged caller data can still be confidential. |
| Exact variant and alphabet/key behavior scattered across code/guides | `profile.variant`, `alphabet`, `key`, `key_advancement` | Reuses the owning description for the variant and states preservation/advancement rules. |
| Digest size/block meaning implicit | `profile.block_bits`, `output_bits` | Uses nullable bit counts; digest compression/absorption blocks do not imply a block cipher. |
| Limits differ across interfaces | `resources[].name`, `maximum`, `unit`, `scope`; `resource_notes` | Records applicable bounds and missing legacy caps explicitly. |
| Direct entry points not named by the descriptor | `rust_api`, `python_api` | Complements the existing CLI command and actual `pipeline` availability. |
| Primary references absent from discovery | `references` | Links specifications/maintainer references without asserting backend certification. |

Annotations cover the implemented `ciphers`, `encoding`, `hashing`, `binary`,
`blockchain` and `symbols` categories and UTF-8 conversions. Other descriptors
omit `metadata` even in an opted-in response. Absence means unannotated, not
unavailable. The complete catalog and executable adapter set are unchanged.
Full invocation/result/error schemas, named ports, generated references and
additional Play execution integration are separate contracts; these descriptive
profile facts do not supply them.

The owning module authors its original descriptor and optional facts in one row.
The public descriptor slice is projected from those rows, and the existing catalog
registration assembles them. There is no extra operation-ID map or dispatcher.
Exact descriptor copies retain facts; a caller-created descriptor with changed
semantics does not inherit facts simply by reusing an ID.

## Interpret profiles precisely

Aliases are descriptive, never lookup identifiers. ROT1–25 use Caesar shifts;
Multiplicative uses Affine with `b=0`; Variant Beaufort encryption uses Vigenère
decoding with the same ASCII-only key advancement. These names do not increase
the operation count. Historical ciphers provide puzzle transformations rather
than modern confidentiality. Encodings provide representations, CRCs detect
accidental changes, and unkeyed hashes alone do not authenticate messages or
provide password hashing. MD5 and SHA-1 have explicit legacy limitations.

SHA-256 and SHA-512 do not imply every SHA-2 variant. Keccak-256 is not standardized
SHA3-256. RIPEMD-160 and CRC-16/IBM-3740 describe exact implemented profiles rather
than their entire families. Backend dependencies do not advertise additional
Cube algorithms. References accompany each annotated profile; examples include
[RFC 4648](https://www.rfc-editor.org/rfc/rfc4648.html),
[NIST's hash specifications](https://csrc.nist.gov/projects/hash-functions), and
[MD5 security considerations](https://www.rfc-editor.org/rfc/rfc6151.html).

Private-key operations remain direct APIs with their existing explicit export
and redaction behavior. Discovery contains static public facts and parameter
names, never caller keys or secret-valued defaults. Metadata grants no execution,
file, network, upload or private-export capability. `pipeline` continues to come
from the executor's real allowlist, independently of annotations.

## Bounds and failures

`resources` scopes are `core`, `cli`, `python` and `pipeline`. A Core bound applies
to every caller of that direct function; an interface may impose a smaller one.
Some entries are mathematical output maxima rather than separate configurable
budgets. Read their names and `resource_notes` together. An absent limit must not
be interpreted as an enforced cap. For example, legacy Caesar has a 1 MiB CLI
input ceiling while its direct Rust/Python calls are caller-sized; older byte
codecs and SHA-256 also retain caller-sized direct inputs. These facts do not
change algorithm behavior or introduce new execution limits.

Discovery itself has a 1 MiB maximum encoded document size and 256-byte UTF-8
query ceiling. Lower the document limit with `--max-output-bytes` or the native
Python keyword `max_output_bytes`; a larger ceiling is rejected. The limit counts
the entire UTF-8 document, including CLI envelopes, indentation and the final
newline (also counted in Python's compact internal JSON before parsing).

Serialization checks before every append. An exact-sized budget succeeds; one
byte less returns `resource_limit` without a partial result/stdout prefix.
Unknown IDs/categories return `invalid_parameter` with operation and parameter
context. Oversized queries return `resource_limit`. Python rejects booleans,
negative values, floats and coercible strings as size budgets. Core serialization
scratch is bounded by `2 * max_output_bytes + 64 KiB` for the current static catalog;
CPython dictionary/string overhead after parsing is outside that encoding budget.
Work is bounded by the static catalog and document cap, with no algorithm calls,
external I/O or caller-supplied parameter expansion.
