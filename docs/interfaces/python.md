# Python SDK

Use Cube from Python scripts and applications. The native SDK calls the Rust
engine directly, without starting a CLI process. It supports the operations in
the [operation reference](../reference/operations.md).

[Install the SDK](../installation.md#python), then import `cube`.

## Bytes and text

```python
import cube

decoded = cube.base64_decode("SGVsbG8=")
print(decoded)
# b'Hello'
print(cube.utf8_decode(decoded))
# Hello
```

Use `bytes` for binary data and `str` for text. Binary results are returned as bytes,
without the CLI's hexadecimal display or an added newline.
See [bytes and text](../concepts/data.md) for the data model.

## Work with images in memory

You can create an image from a byte buffer and inspect it without a temporary file:

```python
import cube

image = cube.Image.from_samples(2, 1, "rgb", b"\xff\x00\x00\x00\x00\xff")
rotated = image.rotate(90)
print(rotated.width, rotated.height)
# 1 2
```

The original image stays 2 × 1; rotation returns a new image. Use `Image.open(path)` to read a file and
`image.save_png(path)` to save one.

## Compose operations

Pipelines can pass typed values between supported operations and retain selected
intermediate results:

```python
import cube

steps = [
    {"operation": "encoding.hex.encode", "version": 1},
    {"operation": "encoding.hex.decode", "version": 1},
]
result = cube.execute_pipeline(steps, b"\x00\xff", capture=[0])
print(result["output"])
# b'\x00\xff'
print(result["intermediates"])
# {0: '00ff'}
```

Direct API support does not imply pipeline support. Use `cube.pipeline_adapters()`
to list compatible operations. See [pipeline rules](../reference/topics/compose-operations.md)
for types, versions and limits.

## Read and write files

File APIs accept strings or `Path` objects and require authorized regular files;
they reject symbolic links. Cube extraction and image-save methods create new
files and reject existing destinations.

Ordinary Python file writes follow Python's own rules. For example,
`Path.write_bytes(...)` overwrites an existing file. Choose the destination
deliberately when saving returned bytes.

## Rank classical candidates

Supply a model and an explicit key list to keep the experiment reproducible:

```python
import cube

model = {
    "kind": "dictionary",
    "entries": [{"text": "A", "weight": 1}],
    "unseen_weight": 0,
}
report = cube.caesar_rank({"text": "B", "shifts": [0, 1], "model": model})
print(report["candidates"][0]["plaintext"], report["status"]["complete"])
# A True
```

The score reflects this caller model; it is not a probability of correctness.
Reports retain key aliases and completion status. Python keeps fixed-point scores
as exact integers. The optional `cancellation=` keyword accepts the existing
`cube.CancellationToken`; analysis releases the GIL so another thread can cancel.
See [classical cryptanalysis](../reference/topics/cryptanalysis.md) for model
semantics, period evidence, transposition search, token observations and crib checks.
For already counted symbols, `cube.frequency_compare` accepts aligned bins and
an explicit expectation policy:

```python
report = cube.frequency_compare({
    "bins": [
        {"label": "A", "observed": 20, "expected": 25},
        {"label": "B", "observed": 30, "expected": 25},
        {"label": "C", "observed": 50, "expected": 50},
    ],
    "expected_policy": "counts",
})
print(report["distance"])
# {'kind': 'finite', 'value': 2.0}
```

This is a frequency distance, not a probability of correctness.

## Discover operations

```python
import cube

description = cube.describe_operation("encoding.base64.decode", include_metadata=True)
print(description["metadata"]["python_api"])
# base64_decode
```

`cube.operations()` lists the installed operations. Where available, optional
metadata identifies the Python entry point. The [Python API signatures](../../bindings/python/python/cube/_native.pyi)
list functions, classes, arguments and return types. Use the
[operation reference](../reference/operations.md) for formats and limits.

## Errors

```python
import cube

try:
    cube.hex_decode("zz")
except ValueError as error:
    print(error.code, error.operation)
# invalid_input encoding.hex.decode
```

Operation validation errors use `ValueError` with `code`, `operation`, `parameter`
and `offset` attributes where available. Pipeline errors add `step` context.
File I/O failures use `OSError`.

Incorrect argument types raise `TypeError`; values outside a binding's numeric
range raise `OverflowError`, such as a Caesar shift outside the signed 32-bit range.
Those argument errors may have no operation-context attributes.

<a id="python-interface"></a>
