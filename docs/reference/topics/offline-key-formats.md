# Offline key formats

## WIF and scalar validation

`blockchain.wif.validate` uses Base58Check, supports mainnet 0x80 and shared
0xef test-family versions, and requires 32 scalar bytes or 32 followed by 0x01.
The marker selects compressed public-key intent; it does not compress a scalar.
The 0xef version cannot distinguish testnet, signet and regtest. An optional
expected network family must match. Unsupported versions, lengths/markers,
checksums and scalars fail explicitly without input contents in diagnostics.

`blockchain.scalar.validate` accepts exactly 32 big-endian raw bytes or 64 ASCII
hex digits (either case, no prefix/whitespace). Maintained pure Rust
[K256](https://docs.rs/k256/0.14.0/k256/) validates `1 <= d < n`; no custom curve
arithmetic or permissive numeric padding is introduced. Private input is capped
at 128 transport bytes before processing. Rules/vectors follow
[Bitcoin Core WIF handling](https://github.com/bitcoin/bitcoin/blob/master/src/key_io.cpp).

Validation returns safe fixed scalar metadata or WIF version/network-family and
compression facts. It never returns WIF, scalar bytes or derived fingerprints.
Rust private input/key debug representations are redacted, and neither operation
has a pipeline adapter or automatic investigation capture. CLI supports exact
stdin; using `--text` instead puts material in argv. Core/SDK calls perform no I/O
or persistence. Interface strings/buffers can retain copies: no process-wide or
Python memory-zeroization guarantee is made. Validation does not export material.

## Bitcoin public keys and P2PKH

`blockchain.bitcoin.derive` accepts declared raw32/hex64/WIF and requires explicit
`mainnet`/`testnet` and compressed/uncompressed encoding. WIF network/compression
intent must agree; neither is silently overridden. K256 provides standard SEC1
bytes (33 compressed or 65 uncompressed). HASH160 is
RIPEMD-160(SHA-256(public-key **bytes**)); P2PKH uses Base58Check versions 0x00/0x6f.
Only public key/digest/address facts are returned. Native SDK fields include raw
`public_key`/`hash160` bytes; CLI JSON uses `public_key_hex`/`hash160_hex` alongside
address/version/encoding/network-family facts. Input remains bounded at 128 bytes,
work is one validated curve multiplication and fixed-size hashes, and no network,
private export or pipeline capture is performed.

Reusable `hash.ripemd160` returns 40 lowercase hex digits of exact bytes with a
16 MiB direct input cap. Its version-1 byte/text pipeline adapter uses the existing
smaller engine budgets; it is separate from private-input derivation. It uses
[RustCrypto RIPEMD](https://docs.rs/ripemd/0.2.0/ripemd/), not a custom primitive.

## Explicit private conversion

`blockchain.private.export` deliberately returns plaintext private material.
Declared raw32/hex64/WIF input uses the same scalar/WIF validators. Output is
exactly 32 raw bytes, 64 lowercase hex digits or canonical WIF, preserving every
scalar byte. WIF output requires explicit network and compression; supplied source
WIF intent must match. Source-WIF expectations are optional for raw/hex output;
context flags on scalar-to-scalar conversion are rejected.

Rust returns a nonserializable result with redacted Debug and an explicit
`into_private_bytes` method. Native `export_private_key` returns caller-owned bytes
or text by an explicitly named export call. CLI additionally requires
`--private-output` and exactly one `--stdout`/`--output` destination. All output is
exact without added newline/metadata. Files use exclusive creation, Unix mode
0600 (other systems use platform ACLs), and failure removes only newly created
partial files. Inputs/existing destinations are never overwritten. Transport/output
is capped at 128 bytes. No automatic persistence, logging, pipeline/capture adapter
or process-wide/Python memory-zeroization guarantee is introduced.

## Ethereum public addresses and Keccak-256

`blockchain.ethereum.derive` accepts exact raw32/hex64 scalars through the shared
secp256k1 validation/public-key implementation. It hashes the 64 uncompressed
**x||y bytes without the SEC1 0x04 prefix** with Ethereum Keccak-256 and takes the
last 20 bytes. Leading zero bytes remain present. WIF is rejected because Bitcoin
network/compression intent has no equivalent here.

The result includes public coordinates, exact address bytes and `0x` text. Choose
`eip55` (default) or `lowercase`. [ERC-55/EIP-55](https://eips.ethereum.org/EIPS/eip-55)
hashes the lowercase 40 ASCII hex digits **without** `0x` to select letter case;
this is not a chain-specific checksum. Known answers include the published ERC-55
cases and a [go-ethereum test vector](https://github.com/ethereum/go-ethereum/blob/master/crypto/crypto_test.go).

`hash.keccak256` is a separate reusable exact-byte digest (<=16 MiB, 64 lowercase
hex digits), also available through a v1 bytes-to-text pipeline adapter.
**Keccak-256 is distinct from standardized SHA3-256.** Address derivation has no
pipeline/capture adapter, network calls, wallet functionality or automatic
persistence. Scalar input handling has the same limits and privacy caveats as
Bitcoin derivation.


## Bounded key candidates

`blockchain.candidates.validate` checks **caller-supplied**, ordered WIF or strict
hex64 entries. Unknown format labels produce a fixed `unsupported` classification;
invalid entries have isolated safe errors. There is no format guessing or candidate
generation. Duplicate entries retain their original zero-based indexes.

Default results contain validity, fixed scalar/WIF facts, safe errors and progress;
they contain no input, private bytes, fingerprint or public address. Explicit
Bitcoin derivation requires network/compression and preserves WIF intent. Explicit
Ethereum derivation accepts scalar hex; a valid WIF remains valid but has a separate
`address_error`, matching the single-entry Ethereum transport restriction. Address
errors never change structural validation counts.

Limits are preflighted before the first result: 256 entries, 128 UTF-8 bytes per
entry and format label, 32 KiB total material, 1,048,576 input-work units, and 32 entries when public
derivation is requested. Work is `(bytes + 2)^2` for WIF conversion and `bytes + 1`
for other entries; curve work has its separate count cap. The conservative JSON
output bound is `2048 + 2048 * entries` bytes, capped at 1 MiB. CLI JSON transport is
at most 64 KiB and reuses duplicate-key rejection with depth 16/node 4096 limits;
only `format` and `value` string fields are accepted.

Rust and Python provide an iterator with a preflight plan and current progress.
`complete` is false until the final entry is consumed (true for an empty batch).
Stopping early does not process the remainder or claim complete counts. The CLI
consumes the bounded batch and returns a complete report; interruption/write
failure is a failed command, not a successful partial report. Malformed schemas,
transport Unicode or resource failures reject the whole batch before processing.
Ordinary invalid candidate content remains an independent result.

There are no pipeline/capture adapters, saved reports, network requests, key search
or ownership/balance claims. Input copies remain caller-owned; returned safe facts
and opt-in public addresses can be persisted only by the caller.
