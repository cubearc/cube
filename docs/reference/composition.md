# Ordered composition

Concatenate two explicit operands without changing their content. Text and bytes
use different operations; choose UTF-8 encoding or decoding explicitly when a
workflow crosses that boundary.

| Operation | Named inputs | Result |
| --- | --- | --- |
| `text.concat` | `left`, `right`: Unicode text | Left text, separator, then right text |
| `bytes.concat` | `left`, `right`: exact bytes | Left bytes followed by right bytes |

Both inputs are required. Empty inputs are valid. Text preserves combining
scalars, NUL, spaces and CRLF without normalization or trimming. Its optional
`separator` defaults to empty text and is included once, even when either operand
is empty. Byte concatenation has no implicit separator or text interpretation.

## CLI examples

```sh
cube compose text --left The --right key --separator ' '
# The key
cube compose bytes --left-hex 00ff --right-hex 800a
# 00ff800a
```

Text composition writes the exact result with no added newline. Binary output
defaults to hexadecimal with a terminating newline; `--output raw` writes exact
bytes, while `--output text` requires valid UTF-8. Invalid hexadecimal or UTF-8
produces an error before output is written.

The Rust functions are `cube_core::composition::concat_text` and `concat_bytes`.
Native Python exposes `cube.concat_text(left, right, separator="")` and
`cube.concat_bytes(left, right)`. Byte values remain `bytes`; the SDK does not
coerce text to bytes.

## Types and limits

The combined operands, including the text separator, must fit **1 MiB**. Their
size is checked with overflow detection before allocating the result. The output
has exactly that byte length. Exceeding the bound returns `resource_limit` with
the operation ID; no partial result is returned. Text bounds count UTF-8 bytes,
rather than visible characters.

Named contracts represent bytes as `{ "base64": "..." }`; they do not accept a
string or numeric array in place of that value. Missing operands, wrong types and
unknown fields are rejected by contract validation. Algorithms and decoding also
enforce their runtime bounds. The contracts grant no filesystem or network access.

These are direct Core operations. Their presence in the catalog does not add a
multi-input adapter to the [linear Core pipeline](../guides/pipelines.md).
The [Play graph editor](https://github.com/cubearc/play/blob/main/docs/explorations.md#combine-image-and-caption-results)
binds each operand to a chosen node output through its own tested native adapters.

See the generated [operation contracts](operations.md) for complete named schemas
and the [text reference](topics/text-operations.md#normalization-and-utf-8) for
explicit UTF-8 conversion.

For structured values and arithmetic, see [fields and exact integer composition](structured-composition.md).
