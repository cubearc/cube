# Binary transformations

These byte transforms accept at most 16 MiB and never interpret text. AND/OR require
equally long operands (empty/empty is valid); no repetition or padding occurs.
NOT flips eight bits per byte. Byte reversal reverses the whole buffer; endian
swapping reverses bytes inside each complete 1/2/4/8-byte unit, keeping unit order.
Both reversals round trip.

Shifts use independent unsigned big-endian 8/16/32/64-bit units (one byte by
default), with no carry across unit boundaries. They zero-fill and discard bits
outside the width. Shift zero is identity, exactly the width yields zero, and
larger shifts or partial units fail. These are fixed-width unit operations, not
signed arithmetic or one unbounded integer over the entire buffer. Repeating-key XOR uses a separate operation, `binary.xor`. `binary.xor_exact` requires equally long
operands, including empty/empty. `binary.rotate_left`/`rotate_right` normalize an
unsigned u64 `count` modulo the unit bit width. `binary.add_wrapping`/`sub_wrapping`
add/subtract equally long buffers modulo 2^width independently, including u64
overflow/underflow; no carry or borrow crosses units. All five functions validate
widths/complete units before processing empty inputs. Each operand is <=16 MiB.
Pipeline adapters use the same functions; binary operand parameters are capped at 64 KiB in pipelines and value-dependent lengths are
checked at execution, while types/parameters/budgets are preflighted.

Use byte-wise repeating-key XOR to inspect a known mask or reverse a challenge
transformation. Applying the same nonempty key twice restores the original bytes.
It is not secure encryption.

```sh
cube binary xor --text 'Hello' --key-hex 6b6579
# 230015070a
```

`--key` supplies UTF-8 key bytes; `--key-hex` supplies arbitrary bytes. Choose exactly
one. Use raw output when chaining binary operations. The descriptor is `binary.xor`.
