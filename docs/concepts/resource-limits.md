# Resource limits

An operation's limits tell you which inputs it accepts and how much processing
and output it permits. Read the operation's own limits table for exact values;
Cube has no single input, memory or work maximum shared by every operation.

## Units and scope

| Unit | Meaning |
| --- | --- |
| KiB | 1,024 bytes |
| MiB | 1,048,576 bytes |
| Mi scalars or Mi work units | 1,048,576 scalars or units |

A decimal limit of 1,000,000 remains one million. A byte limit differs from a
[scalar limit](text.md#bytes-scalars-and-visible-characters).

Check whether a limit applies **per input**, **per item**, or to the **whole
operation**. Two inputs can each fit their byte limit while exceeding a combined
memory or work budget. All applicable limits must hold at the same time.

## Processing work

Work units count steps specified by an operation, such as bytes read, scalars
visited, graph edges examined or candidate comparisons. They are deterministic
accounting units, rather than CPU instructions or a duration in seconds. Work
totals from different algorithms need not measure the same kind of step.

Discarded data can still consume work. With an ASCII-letter filter, the `!` in
`a!` must be inspected before it can be discarded. A small retained result can
therefore require substantial processing.

The operation reference defines the charged steps. Where supported, `max_work`
lowers the processing budget; Play can also impose a worker deadline.

## Memory and report size

**Scratch memory** holds temporary processing state. A **combined memory** budget
can also include inputs, retained results and serialization buffers. The operation
reference defines which allocations count; some APIs exclude an already-owned
input. A Core allocation bound does not measure the entire CLI or Python process,
including its runtime and allocator overhead.

The **report budget** bounds the serialized result, often compact JSON. Escaping,
Base64, field names and wrappers can make a report larger than its raw values.
A report buffer can count toward both a report budget and a combined memory
budget. These budgets are separate constraints on some of the same data.

## Why a request can fail before using the whole allowance

Some operations check a conservative upper bound before processing; this is
called **preflight**. It reserves space for possible output and temporary storage.
The estimate can be larger than the eventual result.

If the estimate exceeds the budget, the request can fail even when a shorter
actual output would have fitted. For example, reducing a requested trace or
output allowance can reduce its reserved memory. Keep the operation's specific
accounting rule when calculating a smaller budget.

Hard ceilings are the highest supported values. Where caller limits are exposed,
they can lower those ceilings. A lower budget does not make the algorithm trim
data silently: the operation either fails or returns the explicitly documented
[partial-result state](results.md).

## Core and interface limits

The CLI, Python binding and Play validate their own transports in addition to
Core's input limits. A JSON request includes more than its raw text or byte
payload, so an interface can reject data that fits the Core algorithm's limit.

Use the installed operation contract and interface reference for exact settings.
[Contributor accounting notes](../resource-accounting.md) document allocation
formulas and internal storage where a detailed bound is needed.
