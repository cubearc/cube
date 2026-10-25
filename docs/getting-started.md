# Getting Started

Use this tutorial to install the CLI, decode data and inspect your first file.
Shell examples use a POSIX shell, such as Bash or Zsh. For Python setup and
Windows virtual-environment activation, use the [Python Guide](python.md#installation).

## Prepare your tools

Install Git, [stable Rust](https://rustup.rs/) and your platform's C linker.
On macOS, the Xcode command-line tools provide the linker; on Windows with the
MSVC Rust toolchain, install the Visual Studio C++ build tools.

## Install the CLI

```sh
git clone https://github.com/cubearc/cube.git
cd cube
cargo install --path crates/cube-cli --locked
cube --version
```

The last command prints the version built from your checkout. If `cube` is not
found, add Cargo's binary directory (normally `$HOME/.cargo/bin`) to `PATH`.
Alternatively, build and run without installing:

```sh
cargo run -p cube-cli -- --version
```

The remaining examples use `cube`; substitute `cargo run -p cube-cli --` if needed,
and keep your working directory inside the checkout when using Cargo.

## Discover an operation

```sh
cube ops list --category encoding
cube ops describe encoding.base64.decode
cube decode base64 --help
```

The description explains the operation's input/output types and gives an example.
Help describes the exact command arguments. Category names are case-sensitive;
`cube ops list --json` shows the categories available in your build.

## Encode and decode

```sh
cube encode base64 --text 'Hello'
# SGVsbG8=
cube decode base64 --text 'SGVsbG8='
# 48656c6c6f
```

The decoder returns bytes. Its default display is hexadecimal: `48` is the byte
for `H`, `65` for `e`, and so on. To write the decoded bytes directly:

```sh
cube decode base64 --text 'SGVsbG8=' --output raw
```

This writes `Hello` without an added newline. For stdin input, preserve the exact
encoded text; a trailing newline is not ignored:

```sh
printf 'SGVsbG8=' | cube decode base64 --output raw
```

## Inspect a local file

Create an original four-byte fixture in a scratch directory:

```sh
printf 'Cube' > sample.bin
cube inspect sample.bin
cube file hex sample.bin
```

Inspection reports `Size: 4 bytes`, filesystem timestamps when available, and no
supported signature candidates. The hex row starts at offset `00000000` and
contains `43 75 62 65`. An unknown signature does not mean a file is invalid.
These commands read the file without changing it.

## Keep binary output intact

Hexadecimal is readable text; raw output may contain NUL or non-UTF-8 bytes.
Save raw bytes to a file instead of displaying them in a terminal:

```sh
cube decode base64 --text 'AP8=' --output raw > decoded.bin
cube file hex decoded.bin --json
```

The JSON row's `hex` is `00ff`: two original bytes, not the four characters
`0`, `0`, `f`, `f`. `--json` is available on inspection commands; it is not a
global output switch. [CLI output handling](cli.md#choose-an-output-format)
explains the conventions for other command groups.

## Recover an embedded file

Run this example in a new scratch directory. Python's standard library creates
an original ZIP fixture; Cube performs the investigation and recovery:

```sh
python - <<'PYTHON'
import io
import zipfile
from pathlib import Path

buffer = io.BytesIO()
with zipfile.ZipFile(buffer, "w") as archive:
    info = zipfile.ZipInfo("note.txt", (1980, 1, 1, 0, 0, 0))
    info.create_system = 3
    info.external_attr = 0o600 << 16
    archive.writestr(info, b"Cube\x00\xff")
encoded = buffer.getvalue()
assert len(encoded) == 120
Path("mystery.bin").write_bytes(b"CubeArc\x00" + encoded + b"\x00tail")
PYTHON
cube inspect mystery.bin --json
cube file hex mystery.bin --length 32 --json
cube file strings mystery.bin --json
cube file embedded mystery.bin --format zip --json
cube file zip mystery.bin --offset 8 --json
cube file carve mystery.bin --offset 8 --output recovered.zip --json
cube file extract mystery.bin --offset 8 --length 120 --output selected-range.zip
```

The host's unknown header is an observation. The PK match at offset 8 is a
candidate; ZIP structure validation establishes the supported archive range
`[8, 128)`, with five trailing bytes. Other PK matches are not automatically
independent valid archives. The report states that payload CRCs and compressed
content were not verified. Recovery revalidates framing, writes a new file and
returns provenance including SHA-256; the explicit range copy produces the same
120 bytes. Existing destinations are rejected and `mystery.bin` is unchanged.
Nothing is decompressed or executed.

For an encoded hint, compose compatible operations and retain the decoded bytes:

```sh
cube pipeline run --text '504b0304' --steps '[{"operation":"encoding.hex.decode","version":1},{"operation":"file.signature.detect","version":1}]' --capture 0 --format json
```

The final typed report contains a ZIP signature candidate; capture 0 contains the
exact decoded bytes. A signature remains evidence, not structural validation.
Use the [Python recovery example](python.md#validate-and-recover-an-embedded-zip)
for the same workflow without parsing terminal text, and the
[Operations Guide](operations.md#validate-a-zip-candidate) for limits and unsupported
ZIP variants.

## Continue

- Use `cube --help` or a nested command's `--help` for arguments.
- Follow the [CLI Guide](cli.md) for binary files, images, audio and workflows.
- Use the [Operations Guide](operations.md) to choose an appropriate transformation.
- Use the [Python Guide](python.md) to work with results in a program.
- Read [Contributing](../CONTRIBUTING.md) for the Rust API reference and development.
