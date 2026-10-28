# Lab capture format

The CLI exports the named result it has already computed. It keeps its normal
stdout output and does not rerun the operation or parse terminal text.

## Contents

A capture contains operation, contract and adapter versions; effective
non-sensitive parameters; typed outputs; schema and content SHA-256 digests;
and input content identities. Binary values use Base64. Images keep their logical
samples and shape.

Content hashes and byte sizes describe the JSON wire values. Schema hashes
describe the installed named-port schemas. Original inputs, source paths,
command lines and terminal history are not retained.

Input identities cannot locate a source or authorize replay. The record identifies
these replay gaps. Valid hashes establish content consistency, not proof that
a command executed.

## Limits and writes

Captures are limited to 128 KiB, 4,096 JSON nodes and 16 levels of nesting.
Serialized inputs are limited to 1 MiB; captured logical images are limited to
64 KiB before Base64 and JSON allocation. Oversized or invalid results fail.
Private or sensitive operations do not support capture.

The destination must be a new file; existing paths and symlinks are rejected.
Write errors return failure, and an incomplete file is not a valid capture.
On Unix, new files have owner-only permissions. Capture does not upload or
automatically import the result.

## Play integration

Inside a configured Play Runner, bare `--lab-capture` writes into the trusted
`CUBE_PLAY_CAPTURE_DIR`. Outside Runner, provide `--lab-capture=PATH`.

Play validates the selected capture before publishing its outputs. See
[Play's capture workflow](https://github.com/cubearc/play/blob/main/docs/terminal.md#send-a-computed-result-to-the-lab)
for staging, import and persistence.

`pipeline run --capture INDEX` is a separate option that retains an intermediate
pipeline value.
