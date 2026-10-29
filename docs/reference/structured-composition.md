# Fields and exact integer composition

Select a JSON value without converting its type, then compose integer results
through explicit operations. No expression language or arbitrary evaluation is
involved. [Play's image and caption example](https://github.com/cubearc/play/blob/main/docs/explorations.md#combine-dimensions-and-a-numeric-caption)
uses these operations in a graph.

| Operation | Inputs | Result |
| --- | --- | --- |
| `json.parse` — Parse JSON | Text containing one JSON document | Structured JSON value |
| `json.select` — Select field | JSON and `pointer` | Selected JSON value, including `null` |
| `json.text` — Use text value | JSON string | Exact text without JSON quotes |
| `text.integer` — Parse integer | Canonical decimal text | Exact integer |
| `number.maximum` — Maximum | Nonempty array of integers | Largest integer |
| `number.add` — Add | Named integers `left`, `right` | Exact sum |
| `image.dimensions` — Image dimensions | Decoded image | `{"dimensions":[width,height]}` in pixels |

## Select fields

The empty pointer selects the root. `/dimensions/0` selects its first array item.
Escape a slash inside an object key as `~1` and a tilde as `~0`; `/a~1b/~0key`
selects `~key` inside `a/b`. Invalid escapes, invalid array indexes and absent
fields fail. An existing `null` is a successful JSON result, distinct from a
missing field. Objects, arrays, Booleans and numbers retain their types.

**Use text value** accepts only a JSON string, preserving empty text, Unicode,
NUL and whitespace. It does not stringify objects, numbers or `null`.

```sh
cube compose parse-json --text '{"clue":"Gur xrl.","empty":null}'
# {"clue":"Gur xrl.","empty":null}
cube compose select --json '{"dimensions":[2,3]}' --pointer /dimensions
# [2,3]
cube compose json-text --json '"Gur xrl."'
# Gur xrl.
```

JSON output is compact with a terminating newline. `json-text` writes the exact
string without adding a newline. Parsing rejects duplicate object keys, trailing
content, invalid syntax and nonfinite numbers.

## Compute with integers

Integers range from **−9,007,199,254,740,991 to 9,007,199,254,740,991**, inclusive,
so they remain exact across Rust, Python and JavaScript. Addition fails if the
sum leaves this interval. Maximum accepts 1–4,096 integers. Empty arrays,
fractional values, floating representations such as `2.0`, strings and Booleans
are rejected by the integer adapters; no rounding or mixed-type coercion occurs.

Parse integer accepts ASCII digits with an optional leading minus. It rejects
spaces, plus signs, leading zeros, decimal points and exponents. `-0` becomes
zero. Explicitly parse caption text before connecting it to Add.

```sh
cube compose integer --text 7
# 7
cube compose maximum --values '[2,3]'
# 3
cube compose add --left 3 --right 7
# 10
```

To inspect a local image, use `cube image dimensions sample.png`. The result lists
width before height. It does not identify letters or interpret image content.

## Bounds and interfaces

JSON documents and serialized constructed values are limited to **256 KiB**,
**16 levels** below the root and **4,096 values**. Object keys and string bytes
also count toward the byte limit. A pointer is at most **1,024 UTF-8 bytes**.
JSON numbers may be fractional but must be finite and have magnitude at most
the integer bound above; fractional values use binary64 representation. Integer
operations apply their stricter policy separately. Limit failures return an
explicit error without a partial result.

Rust functions live in `cube_core::composition::structured`; image inspection is
`cube_core::image::dimensions::inspect`. Native Python exports `parse_json`,
`select_field`, `json_text`, `parse_integer`, `maximum`, `add_integers` and
`image_dimensions`. The first three take JSON **document text**; parsed results
are native Python values. Numeric inputs require builtin integers, excluding
Booleans and floats. `image_dimensions` takes a `cube.Image`.

These are direct Core operations with [named contracts](operations.md), not
additional adapters for the [linear Core pipeline](../guides/pipelines.md).
Play applies its smaller per-Run input/output budgets independently.
