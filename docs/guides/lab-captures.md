# Send a typed CLI result to the Lab


Request `--lab-capture=PATH` for `text reverse`, `decode hex`, `image invert` or
`image histogram`. The command exports its already-computed named result and
keeps its ordinary stdout representation. It never reruns or parses terminal
output. Other commands reject the flag before execution.

```sh
cube --lab-capture=reverse.json text reverse --text 'CUBE'
cube --lab-capture=bytes.json decode hex --text '00ff80' --output raw
cube --lab-capture=invert.json image invert sample.png --channels red --output inverted.png
cube --lab-capture=histogram.json image histogram sample.png --max-colors 100 --json
```

The new local JSON file contains operation/contract/adapter versions, effective
non-sensitive parameters, typed outputs, schema/content SHA-256 digests and
input content identities. Binary values use exact Base64; images retain logical
samples and shape. Content hashes and byte sizes describe the canonical JSON wire
values; schema hashes describe the installed named-port schemas. Original inputs,
source paths, command lines and terminal
history are not retained. Input identities cannot locate a source or authorize
replay; the record includes those replay gaps explicitly. Hashes establish
content consistency, not proof that a command executed.

Captures are explicit local exports, limited to 128 KiB, 4,096 JSON nodes and
16 levels of nesting; serialized inputs are limited to one MiB. Captured logical
images are limited to 64 KiB before Base64/JSON allocation. Oversized or
invalid results fail clearly. Private/sensitive operations are outside this
initial cohort. A destination is never overwritten, including a symlink. File
write errors return failure; an incomplete file is not a valid capture. On Unix,
new files have owner-only permissions. There is no upload or automatic import.

Inside a configured Play Runner, bare `--lab-capture` writes a new JSON file into
the trusted `CUBE_PLAY_CAPTURE_DIR`. Outside that environment, supply `=PATH`.
This flag is separate from `pipeline run --capture INDEX`, which retains its
existing pipeline behavior.
