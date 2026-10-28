# Transform bytes

Apply a mask, swap byte order, or perform arithmetic on fixed-width groups of bytes.
These examples display their results as hexadecimal.

## Swap byte order

```sh
cube binary swap-endian --unit-bytes 2 --text ABCD
# 42414443
```

The input bytes are `41 42 43 44`. Reversing each two-byte unit produces
`42 41 44 43`; the units stay in their original order.

## Apply a mask

```sh
cube binary xor-exact --key-hex ff0f --text AB
# be4d
```

Each input byte is XORed with the corresponding key byte. The key must be the same
length as the input. Use `binary xor` when you intend to repeat a shorter key.

## Rotate or add within a unit

```sh
cube binary rotate-left --count 1 --unit-bytes 2 --text AB
# 8284
cube binary add-wrapping --key-hex 0001 --unit-bytes 2 --text AB
# 4143
cube binary sub-wrapping --key-hex 0001 --unit-bytes 2 --text AB
# 4141
```

Here `AB` is one two-byte unit, `4142`. Arithmetic wraps within the selected width;
it does not carry into the next unit.

See [binary transformations](../reference/topics/binary-transformations.md)
for supported widths, shifts and input limits.

## Transform raw AES blocks

For low-level interoperability with already specified raw AES blocks, provide a
16-, 24- or 32-byte raw key file and an input file whose length is a multiple of
16 bytes. This is unauthenticated and does not define a secure encryption format.

```sh
cube cipher aes encrypt --key-file key.bin --file blocks.bin > encrypted.bin
cube cipher aes decrypt --key-file key.bin --file encrypted.bin > restored.bin
```

Use binary-preserving shell redirection; stdout contains bytes, not hexadecimal.
No padding is added or removed. See [raw AES blocks](../reference/topics/aes-blocks.md)
for limits, the public test vector and key privacy.

<a id="cli"></a>
<a id="fixed-width-binary-arithmetic"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="transform-binary-bytes"></a>
<a id="transform-exact-bytes"></a>
