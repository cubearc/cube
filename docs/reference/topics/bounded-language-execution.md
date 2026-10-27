### Bounded language execution

`esolang.brainfuck.execute` runs the [strict Brainfuck profile](../../brainfuck.md) and
returns exact bytes only after successful completion. `esolang.brainfuck.report`
returns an explicitly completed/incomplete execution with optional final tape and
bounded traces. `esolang.brainfuck.compare` runs a selected subset of 8/16/32-bit
cells with aggregate budgets. These direct Rust/CLI/native Python operations have
no pipeline adapters, host effects or arbitrary-width/nonwrapping mode.
