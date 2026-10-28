# Inspect a local file

Create a four-byte sample in a scratch directory, then inspect it:

```sh
printf 'Cube' > sample.bin
cube inspect sample.bin
cube file hex sample.bin
cube file strings sample.bin --json
```

Inspection reports four bytes. The hex row starts at offset zero with
`43 75 62 65`, and the string scan finds `Cube`.

An unknown signature means that no recognized header was found. It does not mean
the file is invalid. These commands read the source without changing it.

Continue with [recover an embedded ZIP](recover-zip.md), or see the
[file reference](../reference/topics/file-forensics.md) for range selection,
signatures and processing limits.

<a id="python"></a>
