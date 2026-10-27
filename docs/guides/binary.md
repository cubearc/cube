# Transform exact bytes

Use byte inputs and an explicit unit width for fixed-width transformations. Raw output remains bytes; hexadecimal is a display representation.

## CLI

### Transform binary bytes

```sh
cube binary and --key-hex ff0f --text AB
cube binary shift-left --bits 1 --unit-bytes 2 --text AB
cube binary swap-endian --unit-bytes 2 --text ABCD
```


```sh
cube binary rotate-left --count 1 --unit-bytes 2 --text AB
# 8284
cube binary add-wrapping --key-hex 0001 --unit-bytes 2 --text AB
# 4143
cube binary sub-wrapping --key-hex 0001 --unit-bytes 2 --text AB
# 4141
cube binary xor-exact --key-hex ff0f --text AB
# be4d
```


## Python

### Fixed-width binary arithmetic

```python
import cube

assert cube.rotate_left(bytes.fromhex("8001"), 1, unit_bytes=2) == b"\x00\x03"
assert cube.rotate_right(b"\x03", 1) == b"\x81"
assert cube.add_wrapping(bytes.fromhex("ffff0001"), bytes.fromhex("00010001"), unit_bytes=2) == bytes.fromhex("00000002")
assert cube.sub_wrapping(b"\x00", b"\x01") == b"\xff"
assert cube.xor_exact(bytes.fromhex("00ff"), bytes.fromhex("ff0f")) == bytes.fromhex("fff0")
```


## Reference

- [Binary transformations](../reference/topics/binary-transformations.md)
