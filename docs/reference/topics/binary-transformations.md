# Binary transformations

Apply bitwise masks, reorder bytes, or calculate with fixed-width unsigned integers.
These operations return bytes and do not interpret text. For a walkthrough, see
[Transform bytes](../../guides/binary.md).

## Bitwise operations

Choose from these bitwise operations:

| Operation | CLI command |
| --- | --- |
| AND | `binary and` |
| OR | `binary or` |
| Exact XOR | `binary xor-exact` |
| Repeating-key XOR | `binary xor` |
| NOT | `binary not` |

- AND, OR and exact XOR combine equal-length operands, including two empty inputs.
  They do not repeat or pad an operand.
- Repeating-key XOR cycles a nonempty key across the input. An empty key fails
  even when the input is empty.
- NOT flips every bit of its single input.

## Byte order

The two reorder operations differ in scope:

| Operation | CLI command | Result for `41 42 43 44` |
| --- | --- | --- |
| Reverse the buffer | `binary reverse` | `44 43 42 41` |
| Swap two-byte units | `binary swap-endian --unit-bytes 2` | `42 41 44 43` |

Both operations restore the input when applied twice. Swapping byte order keeps
the units in their original order.

## Fixed-width arithmetic

Shifts, rotations and wrapping arithmetic treat each unit as an independent,
unsigned big-endian integer. No carry or borrow crosses a unit boundary.

| Setting | Default | Accepted values |
| --- | --- | --- |
| `--unit-bytes` | `1` | `1`, `2`, `4` or `8` bytes |
| `--bits` for shifts | Required | `0` through the unit's bit width |
| `--count` for rotations | Required | Unsigned 64-bit integer; reduced modulo the bit width |
| `--key-hex` for addition/subtraction | Required | An equally long operand containing complete units |

The operations handle overflow as follows:

| Operation | CLI commands | Behavior |
| --- | --- | --- |
| Logical shift | `shift-left`, `shift-right` | Fill with zeros and discard bits outside the unit. |
| Rotation | `rotate-left`, `rotate-right` | Wrap shifted bits around within the unit. |
| Wrapping arithmetic | `add-wrapping`, `sub-wrapping` | Calculate modulo `2^(unit_bytes × 8)`, including 64-bit overflow and underflow. |

A shift of zero leaves the input unchanged. A shift equal to the unit's bit width
produces zeros; a larger shift fails. Partial units fail without padding.
Unit widths and operand lengths are validated even for empty input.

## Limits

Limits depend on the operation and execution interface:

| Scope | Limit |
| --- | --- |
| Bitwise, byte-order and arithmetic operations, except repeating-key XOR | 16 MiB per operand |
| Repeating-key XOR through direct Rust/Python calls | No fixed input or allocation budget; callers control buffer sizes |
| Binary operand parameter in a pipeline | 64 KiB |

Pipelines validate types, parameters and budgets before execution. Operand lengths
that depend on an earlier result are checked when that step runs. The shared
[pipeline limits](compose-operations.md#limits) also apply.

## Repeating-key XOR example

Use `--key` for UTF-8 key text or `--key-hex` for arbitrary key bytes.
Provide exactly one:

```sh
cube binary xor --text 'Hello' --key-hex 6b6579
# 230015070a
```

Applying the same key twice restores the original bytes. Repeating-key XOR is
useful for reversible masks, but it does not provide secure encryption.

Use `--output raw` when passing binary output to another operation.
For the complete schemas, see the [operation reference](../operations.md).
