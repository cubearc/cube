# Python interface

[Install the native SDK](../installation.md#python) before using these examples.
It calls Rust directly; no CLI subprocess is required.

## Bytes and text

```python
import cube

encoded = cube.base64_encode(b"Hello")
assert encoded == "SGVsbG8="
decoded = cube.base64_decode(encoded)
assert decoded == b"Hello"
assert cube.utf8_decode(decoded) == "Hello"
```

Use `bytes` for binary data and `str` for encoded representations and text.
Conversion is explicit; binary results have no hexadecimal display layer or
added newline. See [the data model](../concepts/data.md).

```python
from pathlib import Path

Path("decoded.bin").write_bytes(cube.base64_decode("AP8="))
```

`Path.write_bytes` is a Python filesystem action and overwrites an existing path.
Choose a new destination. Cube extraction/image-save methods create files exclusively.
File APIs accept strings or `Path` objects and require authorized regular files;
they reject symbolic links.

## Discover operations

```python
operations = cube.operations()
description = cube.describe_operation("encoding.base64.decode")
assert description["id"] == "encoding.base64.decode"
```

The installed metadata and [Python stubs](../../bindings/python/python/cube/_native.pyi)
describe available entry points. See [task guides](../guides/index.md) for examples.

## Errors


```python
import cube

try:
    cube.hex_decode("zz")
except ValueError as error:
    assert error.code == "invalid_input"
    assert error.operation == "encoding.hex.decode"

try:
    cube.inspect_file("missing-cube-example.bin")
except OSError as error:
    assert error.code == "io"
```

Operation validation errors use `ValueError` with `code`, `operation`, `parameter`
and `offset` attributes where available. Pipeline errors add `step` context.
File I/O failures use `OSError` in the file APIs. Incorrect Python argument types
raise `TypeError`; integers outside a binding's numeric range raise `OverflowError`
(for example a Caesar shift outside signed 32-bit range). These Python argument
errors need not have operation-context attributes. Correct invalid input or choose
a smaller selection when a resource limit is reported.
