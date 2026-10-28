# Compare a hash or checksum

Use a hash to compare exact caller-supplied bytes. A checksum is useful for
corruption checks; it is not an authentication mechanism.

## CLI

```sh
cube hash sha256 'Hello'
# 185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969
```

## Python

```python
import cube

assert cube.sha256(b"Hello") == (
    "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969"
)
```

MD5 and SHA-1 support legacy comparisons and are unsuitable for collision-resistant
security. See [hash variants and checksum profiles](../reference/topics/hashing-and-checksums.md).
