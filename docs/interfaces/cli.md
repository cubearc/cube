# CLI interface

Install via [Installation](../installation.md), then choose a [task](../guides/index.md).
Shell examples use POSIX syntax unless marked PowerShell. Run output-producing examples in a scratch directory.

## Find the right command

Commands generally follow `cube <group> <operation> [arguments]`.
Use the groups for the task at hand:

- `encode`, `decode`, `cipher`, `hash`, `binary`, `text`, `number`: transform data.
- `inspect`, `file`, `image`, `audio`: inspect files or export selected data.
- `pipeline`, `recipe`: compose operations into repeatable linear workflows.
- `esolang`: run bounded Brainfuck and inspect execution/width comparisons.
- `logic`, `graph`: analyze explicit Boolean formulas and finite multigraphs.
- `investigate`: suggest inspections, record evidence and replay local analysis.
- `ops`: discover operations and their descriptions.

`cube inspect path` is a top-level shortcut for basic file information.
Use `cube --help`, then progressively narrower help such as
`cube image --help` and `cube image lsb --help`.


## Discover operations

```sh
cube ops list
cube ops list --category file
cube ops list --json
cube ops describe file.signature.detect --json
```

Descriptors provide stable operation identifiers, categories, semantic input/output
types, parameters, constraints, defaults, determinism, reversibility and examples.
The JSON listing is sorted by identifier and uses `schema_version: 1`. Consumers
should tolerate added fields and operation identifiers. Each JSON descriptor adds
`pipeline: {"version": 1}` when a bounded adapter exists, or `pipeline: null` when
it does not. Human `ops describe` also reports this availability. The version is
the adapter contract required by a step, independent of the package version.
Unknown categories or identifiers are errors. `pipeline adapters` lists the same
executable subset; parameter, type and resource validation still applies.

Use `--json --metadata` for optional canonical names, qualified aliases, security
limitations and exact profile/resource facts. Existing JSON is unchanged by
default. Discovery output is capped at 1 MiB; `--max-output-bytes` can lower it.
See [profile discovery](../discovery.md) for compatibility, bounds and examples.


## Supply text or bytes

Commands with inline text/byte input accept one source. Quote literal text
containing spaces or shell metacharacters:

| Source | Form |
| --- | --- |
| Literal UTF-8 text | `cube encode base64 'Hello'` |
| Explicit inline text | `cube encode base64 --text 'Hello'` or `-t 'Hello'` |
| File contents | `cube encode base64 --file message.txt` or `-f message.txt` |
| Standard input | `printf 'Hello' \| cube encode base64` |

`--input PATH` is an alias for `--file PATH` on these commands. Positional TEXT,
`--text` and `--file` are mutually exclusive; no selector reads stdin until EOF.
An empty quoted argument is an explicit empty input. Use `--` before positional
text that begins with a dash, for example `cube text reverse -- '-literal'`.

```sh
cube cipher vigenere 'ATTACKATDAWN' -k LEMON
printf 'Cube' > sample.bin
cube encode hex -f sample.bin
cube hash sha256 --file sample.bin
cube encode hex < sample.bin
```

`-k` is the short spelling of `--key` for Vigenere, Beaufort and repeating-key
XOR. These key parameters remain explicit; input data is a separate argument.

Encoding, hashing and XOR accept arbitrary bytes. Text operations and classical
ciphers require valid UTF-8. Files and stdin preserve every byte, including
newlines; use `printf` instead of `echo` when a final newline is not wanted.
A positional value or `--text` value is always literal, even if it names an
existing file. Use `--file` to read its contents. File input is read-only and
must be a regular file; symbolic links, directories and devices fail. Existing
operation-specific byte limits also apply to file and positional input.
File/image/audio inspection commands continue to take explicit path arguments.

Pipeline run also accepts positional TEXT or `--text`/`-t` as typed text.
`--input PATH`, `--file PATH` and `-f PATH` select raw bytes from a regular file;
`--hex` and `--value` retain their explicit representations. Choose exactly one
selector; the default is raw stdin. Recipes instead require explicit named
bindings; see [recipe files](../reference/topics/compose-operations.md#recipe-files).


## Choose an output format

- Ordinary text results have a final newline.
- Decoding, XOR and `text to-bytes` default to hexadecimal plus a newline;
  `--output raw` writes exact bytes without a newline.
- Inspection commands offer human-readable output and, where listed in help,
  `--json`. Some export commands require a destination choice.
- Pipeline/recipe runs use `--format raw|hex|json`. The default displays bytes as
  hex and text as text; JSON preserves type tags and captured intermediate results.

```sh
cube decode base64 --text 'SGVsbG8='
# 48656c6c6f
cube decode base64 --text 'AP8=' --output raw > decoded.bin
cube inspect decoded.bin --json
```

Do not convert arbitrary raw output through a text filter or text-mode file API.
If your shell's binary redirection behavior is uncertain, use the Python SDK's
`bytes` results and `Path.write_bytes` instead.


## Write results to files

`--output` has command-specific meanings: for a decoder it selects representation;
for extraction/transform commands it names a destination. Help is authoritative.

```sh
cube file extract sample.bin --offset 0 --length 4 --output selected.bin
cube file extract sample.bin --offset 0 --length 4 --stdout > selected-stdout.bin
```

File, image and audio exports use `--output path` or explicit raw `--stdout` where
supported. A file destination usually leaves metadata/status on stdout; `--stdout`
emits bytes only. Image transforms require a new PNG destination. Investigation
records use `--save path`. These destination options never overwrite existing
files, including aliases of the source.

Shell `>` redirection is controlled by your shell and can overwrite an existing
file before Cube runs. Use a new path or Cube's exclusive destination option.


## Handle errors and exit codes

Results go to stdout; failures go to stderr. Exit codes are:

- `0`: successful operation, help/version, a comparison finding differences,
  or a valid investigation record (which may say `partial`).
- `1`: invalid operation input, resource/format failure, recipe mismatch or I/O failure.
- `2`: invalid command usage, such as missing arguments or an unknown flag.

A downstream closed pipe exits successfully. To request machine-readable errors:

```sh
cube --json-errors decode hex --text zz
```

This intentionally exits `1` with a `schema_version: 1` JSON error on stderr.
Fields include `code`, `message`, `operation`, `parameter` and byte `offset`;
unavailable context is `null`. Pipeline errors also identify the zero-based
`step`. Codes include `invalid_input`, `invalid_parameter`, `unsupported_format`,
`resource_limit`, `cancelled`, `io`, and CLI-only `invalid_usage`.
