# Inspect a local file

Create an original four-byte fixture in a scratch directory:

```sh
printf 'Cube' > sample.bin
cube inspect sample.bin
cube file hex sample.bin
cube file strings sample.bin --json
```

Inspection reports four bytes; the hex row starts at offset zero with
`43 75 62 65`. An unknown signature is an observation, not an invalid-file verdict.
These commands read the source without changing it.

## Python

```python
import cube

info = cube.inspect_file("sample.bin")
assert info["size"] == 4
```

Use the [Python interface](../interfaces/python.md) for return types and errors.
For embedded archive evidence, continue with [recover an embedded ZIP](recover-zip.md).
The [file reference](../reference/topics/file-forensics.md) covers signatures,
boundaries, range selection, structural validation and processing limits.
