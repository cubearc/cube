# Offline key formats

Validate private-key representations, derive public addresses or explicitly
convert private material. These operations run offline, without wallet access,
network requests or automatic persistence.

## WIF and scalar validation

| Operation | Accepted input | Validation |
| --- | --- | --- |
| `blockchain.wif.validate` | Base58Check WIF | Version, scalar, checksum and optional compression marker |
| `blockchain.scalar.validate` | Exactly 32 big-endian bytes or 64 ASCII hex digits | `1 <= d < n` for secp256k1 |

Hex accepts either case, with no prefix or whitespace. Private input is limited
to 128 transport bytes before processing. Validation uses pure Rust
[K256](https://docs.rs/k256/0.14.0/k256/) without numeric padding or custom curve
arithmetic.

### WIF rules

| Field | Rule |
| --- | --- |
| Version | `0x80` mainnet or `0xef` test family |
| Payload | 32 scalar bytes, optionally followed by `0x01` |
| Compression marker | Selects compressed public-key intent; does not compress the scalar |
| Test family | Cannot distinguish testnet, signet and regtest |
| Expected network | Optional; must match when supplied |

Unsupported versions, lengths, markers, checksums and scalars fail without
including input contents in diagnostics. Rules follow
[Bitcoin Core WIF handling](https://github.com/bitcoin/bitcoin/blob/master/src/key_io.cpp).

### Validation output and private input

Validation returns fixed scalar metadata or WIF version, network-family and
compression facts. It returns no private bytes, WIF text or derived fingerprint.

- Rust private input/key debug output is redacted.
- There is no pipeline adapter or automatic investigation capture.
- CLI stdin accepts exact input; `--text` puts the material in process arguments.
- Core and SDK calls perform no I/O or persistence.
- Interface buffers can retain copies; no process-wide or Python memory-zeroization
  guarantee is made.

## Bitcoin public keys and P2PKH

`blockchain.bitcoin.derive` accepts declared raw32, hex64 or WIF input. Select
`mainnet` or `testnet` and compressed or uncompressed encoding explicitly.
Source WIF intent must agree with both choices.

| Stage | Representation |
| --- | --- |
| Public key | SEC1: 33 compressed or 65 uncompressed bytes |
| HASH160 | RIPEMD-160(SHA-256(public-key bytes)) |
| P2PKH address | Base58Check version `0x00` mainnet or `0x6f` testnet |

Results contain public key, digest and address facts only. SDK fields include
`public_key` and `hash160` bytes; CLI JSON uses `public_key_hex` and `hash160_hex`,
plus address, version, encoding and network-family facts.

Input is limited to 128 bytes. Work is one validated curve multiplication and
fixed-size hashes. There is no private export or pipeline capture.

## Explicit private conversion

`blockchain.private.export` returns **plaintext private material**. It validates
declared raw32, hex64 or WIF input and preserves every scalar byte.

| Output | Representation | Required context |
| --- | --- | --- |
| Raw | Exactly 32 bytes | Source-WIF expectations optional |
| Hex | Exactly 64 lowercase digits | Source-WIF expectations optional |
| WIF | Canonical Base58Check text | Explicit network and compression; source WIF intent must match |

Context flags are rejected for scalar-to-scalar conversions.

### Export interfaces

| Interface | Explicit export |
| --- | --- |
| Rust | Nonserializable result, redacted Debug, `into_private_bytes` |
| Python | `export_private_key` returns caller-owned bytes or text |
| CLI | Requires `--private-output` and exactly one `--stdout` or `--output` |

Output has no added newline or metadata. Files are created exclusively with Unix
mode 0600, or platform ACLs elsewhere. Existing files and inputs are never
overwritten; failure removes only a newly created partial file.

Input and output are limited to 128 bytes. Export adds no logging, automatic
persistence, pipeline/capture adapter or memory-zeroization guarantee.

## Ethereum public addresses and Keccak-256

`blockchain.ethereum.derive` accepts raw32 or hex64 scalars through the same
secp256k1 validator. WIF is rejected because its Bitcoin network and compression
intent has no equivalent here.

Address derivation:

1. Take the 64 uncompressed `x || y` public-key bytes, without the SEC1 `0x04` prefix.
2. Hash with Ethereum Keccak-256.
3. Keep the last 20 bytes, preserving leading zeros.

The result includes public coordinates, exact address bytes and `0x` text.

| Address style | Behavior |
| --- | --- |
| `eip55` (default) | Hash lowercase 40-digit ASCII hex without `0x` to select letter case |
| `lowercase` | Lowercase hex |

EIP-55 is not a chain-specific checksum. Vectors follow
[ERC-55/EIP-55](https://eips.ethereum.org/EIPS/eip-55) and
[go-ethereum](https://github.com/ethereum/go-ethereum/blob/master/crypto/crypto_test.go).

Scalar limits and privacy rules match Bitcoin derivation. Derivation has no
pipeline/capture adapter, wallet functions or automatic persistence.

### Reusable digest operations

These digest functions are separate from private-input derivation:

| Operation | Output | Direct input limit | Pipeline |
| --- | --- | --- | --- |
| `hash.ripemd160` | 40 lowercase hex digits | 16 MiB | Version-1 bytes to text |
| `hash.keccak256` | 64 lowercase hex digits | 16 MiB | Version-1 bytes to text |

Pipeline calls use the smaller engine budgets. RIPEMD-160 uses
[RustCrypto RIPEMD](https://docs.rs/ripemd/0.2.0/ripemd/).
**Keccak-256 differs from standardized SHA3-256.**

## Bounded key candidates

`blockchain.candidates.validate` checks an ordered list of caller-supplied WIF or
strict hex64 entries. It does not guess formats or generate candidates.
Duplicates retain their original zero-based indexes.

### Classification and optional addresses

| Result | Behavior |
| --- | --- |
| Default | Validity, fixed scalar/WIF facts, safe errors and progress |
| Unknown format | Fixed `unsupported` classification |
| Invalid entry | Isolated safe error; other entries still validate |
| Bitcoin derivation | Requires explicit network/compression and preserves WIF intent |
| Ethereum derivation | Accepts scalar hex; valid WIF retains validity with a separate `address_error` |

Default results contain no input, private bytes, fingerprint or public address.
Address errors do not change structural validation counts.

### Candidate limits

All limits are checked before the first result.

| Resource | Maximum or charge |
| --- | --- |
| Entries | 256; 32 when public derivation is requested |
| Each entry / format label | 128 UTF-8 bytes |
| Total material | 32 KiB |
| Input work | 1,048,576 units |
| WIF work | `(bytes + 2)^2` per entry |
| Other input work | `bytes + 1` per entry |
| Conservative JSON output | `2048 + 2048 * entries` bytes; capped at 1 MiB |
| CLI JSON | 64 KiB, depth 16, 4,096 nodes |

CLI entries accept only `format` and `value` string fields; duplicate keys fail.
Malformed schemas, transport Unicode and resource failures reject the whole batch
before processing. Ordinary invalid candidate content remains a per-entry result.

### Progress and retention

Rust and Python expose an iterator with a preflight plan and progress.
`complete` becomes true only after the final entry is consumed; an empty batch
is already complete. Stopping early leaves the remainder unprocessed.

The CLI consumes the batch and returns a complete report. Interruption or write
failure fails the command rather than returning a successful partial report.

No reports are saved automatically. There are no pipeline/capture adapters,
key searches or ownership/balance claims. The caller controls input copies and
persistence of returned facts or explicitly requested public addresses.
