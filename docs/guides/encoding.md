# Encode and decode data

Use an encoding to represent exact bytes as text, or decode to recover them.
The [encoding reference](../reference/topics/encoding-and-decoding.md) defines
alphabets, padding and strict input rules.

## CLI

```sh
cube encode base64 'Hello'
# SGVsbG8=
cube decode base64 'SGVsbG8=' --output raw
# Hello (without an added newline)
```

Use `--file PATH` for file contents; a positional filename is literal text.
See [CLI input and output](../interfaces/cli.md).

## Python

```python
import cube

assert cube.base64_encode(b"Hello") == "SGVsbG8="
assert cube.base64_decode("SGVsbG8=") == b"Hello"
assert cube.hex_decode("00FF") == b"\x00\xff"
```

## Versioned Base58Check

Choose an explicit version and payload. The checksum detects corruption; it
does not authenticate the sender. Plain Base58 has no Base58Check verification.
See [exact Base58Check rules](../reference/topics/encoding-and-decoding.md#versioned-base58check).
