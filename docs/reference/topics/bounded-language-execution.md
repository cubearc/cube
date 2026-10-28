# Bounded language execution

Run Brainfuck with explicit resource limits and no host effects.
The [strict execution profile](../../brainfuck.md) defines the grammar and budgets.

## Operations

| Operation | Result | Completion behavior |
| --- | --- | --- |
| Execute | Exact output bytes | Returns bytes only after successful completion |
| Report | Completion status, optional final tape and bounded traces | Distinguishes completed from incomplete execution |
| Compare | Results for selected 8/16/32-bit cell widths | Applies aggregate budgets across the runs |

These operations have no pipeline adapters, arbitrary-width cells or nonwrapping mode.
See [Run a Brainfuck program](../../guides/brainfuck.md) for an example.
