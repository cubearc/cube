# Install Cube from source

Use Git, stable Rust and a C linker. Install Rust through [rustup](https://rustup.rs/).
On Windows, use the Visual Studio C++ build tools.
These instructions build the checked-out source; they do not assume published packages.

## CLI

```sh
git clone https://github.com/cubearc/cube.git
cd cube
cargo install --path crates/cube-cli --locked
cube --version
```

Keep Cargo's binary directory (normally `$HOME/.cargo/bin`) on `PATH`.
To run from the checkout without installing, use
`cargo run -p cube-cli -- --version` and substitute
`cargo run -p cube-cli --` for `cube` in the examples.

Continue with [your first operation](getting-started.md).

## Python

Python 3.10+ is also required. From the Cube repository root:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install .
```

On Windows, create and activate the environment in PowerShell instead:

```powershell
py -m venv .venv
.\.venv\Scripts\Activate.ps1
```

Then use the same `python -m pip` commands. The distribution is named
`cubearc-cube`; import it as `cube`.

```python
import cube

assert cube.base64_decode("SGVsbG8=") == b"Hello"
```

See [Python types and errors](interfaces/python.md) and
[Contributing](../CONTRIBUTING.md#setup) for editable development.

## Rust

Use the [Rust interface](interfaces/rust.md) for a local path dependency and
the generated API reference. Core is not assumed to be published on crates.io.
