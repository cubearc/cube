# Run a bounded Brainfuck program

Choose a strict program, byte input and explicit limits. Cell width affects wrapping arithmetic; inspect reports before comparing results.

## CLI

### Run bounded Brainfuck

```sh
printf ',.' > program.bf
printf 'A' > input.bin
cube esolang brainfuck run --code '++++++++[>++++++++<-]>+.' --output text
# A (no appended newline)
cube esolang brainfuck run --code ',.' --input-hex ff --output raw > echoed-byte.bin
cube esolang brainfuck run --code-file program.bf --input-file input.bin
cube esolang brainfuck report --code '-.' --cell-bits 16 --tape-len 1
cube esolang brainfuck compare --code '-.' --widths 8,16,32
```


## Python

### Bounded Brainfuck

```python
import cube

assert cube.brainfuck_execute("++++++++[>++++++++<-]>+.") == b"A"
assert cube.brainfuck_execute(",.", b"\x00") == b"\x00"
r = cube.brainfuck_report(
    "-.", options={"cell_bits": 32},
    diagnostics={"tape_len": 1, "max_trace_records": 2, "max_trace_bytes": 1000},
)
assert r["completed"] and r["output"] == b"\xff"
assert r["tape"] == [{"index": 0, "value": 4294967295}]
c = cube.brainfuck_compare("-.", options={"widths": [8, 16, 32]})
assert c["all_completed"] and c["equal_completed_outputs"] is True
```


## Reference

- [Bounded language execution](../reference/topics/bounded-language-execution.md)

- [Execution profiles and limits](../brainfuck.md)
