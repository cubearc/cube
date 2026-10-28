# Run a Brainfuck program

Run a short program and choose how to display its output:

```sh
cube esolang brainfuck run --code '++++++++[>++++++++<-]>+.' --output text
# A (no appended newline)
cube esolang brainfuck run --code ',.' --input-hex ff --output raw > echoed-byte.bin
```

The first program emits `A`. The second reads one byte and writes it back;
`echoed-byte.bin` contains `ff`.

## Compare cell widths

```sh
cube esolang brainfuck report --code '-.' --cell-bits 16 --tape-len 1
cube esolang brainfuck compare --code '-.' --widths 8,16,32
```

Decrementing a zero cell wraps to its largest value. That value depends on the
cell width, even when the emitted low byte is the same. Inspect completion and
resource-limit results before comparing output.

See [execution profiles and limits](../brainfuck.md) and the
[operation reference](../reference/topics/bounded-language-execution.md)
for input rules and diagnostics.

<a id="bounded-brainfuck"></a>
<a id="cli"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="run-a-bounded-brainfuck-program"></a>
<a id="run-bounded-brainfuck"></a>
