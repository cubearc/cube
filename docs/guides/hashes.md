# Compare a hash or checksum

A hash lets you compare the bytes of two inputs. For example:

```sh
cube hash sha256 'Hello'
# 185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969
```

Letter case, whitespace and a trailing newline all change the input.
Use `--file PATH` to hash file contents instead of a text argument.

Checksums help detect corruption; they do not authenticate a sender.
MD5 and SHA-1 are available for legacy comparisons and are unsuitable when
collision resistance is required.

See [hashes and checksums](../reference/topics/hashing-and-checksums.md)
for supported algorithms and profiles.

<a id="cli"></a>
<a id="python"></a>
