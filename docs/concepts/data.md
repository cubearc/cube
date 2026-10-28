# Bytes, text and operations

## Bytes and their display

Bytes are exact binary values. An encoding represents them as text; decoding
returns bytes. A display such as `00ff` represents two bytes, not four text
characters. Select raw output when writing the original bytes.

## Text

Text operations use UTF-8. Read [Unicode and text preparation](text.md) for the
difference between bytes, scalars and visible characters, and for normalization
and filtering. Each operation specifies which preparation it supports.

## Operations, pipelines and Play

An operation has a stable ID such as `encoding.base64.decode`.
Its contract describes accepted input, output, parameters, errors and limits.
Inspect the installed build with `cube ops describe`.

A direct call runs one operation through its Rust, CLI or Python interface.
A pipeline connects compatible operation adapters; only operations with a pipeline
adapter can become pipeline steps. In discovery, `pipeline: null` means there is
no such adapter. A structured report can still be useful through a direct call.

Play advertises its own execution availability and interface budgets. A Core
operation's presence in the catalog grants no filesystem access or Play execution
permission. Inspect the installed build rather than relying on a fixed catalog
or adapter count.

## Recipes and challenges

A recipe saves a versioned pipeline with explicit input bindings. A challenge is
an activity; it may use recipes or Play stages. See
[pipelines and recipes](../guides/pipelines.md) for a working example.

## Limits and result meaning

[Resource limits](resource-limits.md) explains units, work budgets and memory
estimates. [Complete and partial results](results.md) explains how to distinguish
a complete answer, an omitted listing and an interrupted search.
