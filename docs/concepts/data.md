# Bytes, text and operations

Bytes are exact binary values. An encoding represents them as text; decoding
returns bytes. A display such as `00ff` represents two bytes, not four text
characters. Select raw output when writing the original bytes.

Text operations use UTF-8 and document their alphabet or Unicode scalar rules.
A scalar is not necessarily a complete visible character. Normalization,
trimming and interpretation happen only when the operation explicitly supports them.

An operation has a stable ID such as `encoding.base64.decode`.
Its contract describes accepted input, output, parameters, errors and limits.
Inspect the installed build with `cube ops describe`.

A pipeline connects compatible operation adapters. An operation may support a
direct Rust/CLI/Python call without a pipeline adapter. A descriptor also does not
grant filesystem access or enable a Play execution.

A recipe saves a versioned pipeline with explicit input bindings. A challenge is
an activity; it may use recipes or Play stages. See
[pipelines and recipes](../guides/pipelines.md) for a working example.
