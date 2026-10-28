# Encode and decode data

Represent bytes as text with an encoding, then decode to recover the original bytes:

```sh
cube encode base64 'Hello'
# SGVsbG8=
cube decode base64 'SGVsbG8=' --output raw
# Hello (without an added newline)
```

Without `--output raw`, decoded bytes display as hexadecimal. Use `--file PATH`
to read a file; a positional filename is treated as text.
See [CLI input and output](../interfaces/cli.md) for other input methods.

## Versioned Base58Check

Base58Check includes a version and checksum. Choose the version expected by the
format you are reading. Its checksum detects corruption but does not authenticate
the sender; plain Base58 has no such check.

The [encoding reference](../reference/topics/encoding-and-decoding.md) lists the
supported alphabets, padding rules and Base58Check formats.

<a id="cli"></a>
<a id="python"></a>
