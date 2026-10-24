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

## Continue

- Use `cube --help` or a nested command's `--help` for arguments.
- Follow the [CLI Guide](cli.md) for binary files, images, audio and workflows.
- Use the [Operations Guide](operations.md) to choose an appropriate transformation.
- Use the [Python Guide](python.md) to work with results in a program.
- Read [Contributing](../CONTRIBUTING.md) for the Rust API reference and development.
