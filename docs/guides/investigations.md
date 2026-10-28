# Record and replay an investigation

Save an inspection record so you can review its findings and repeat the checks.
Create a small input in a scratch directory:

```sh
printf 'Cube' > sample.bin
cube investigate triage sample.bin --json
cube investigate plan sample.bin --depth preview --json
```

Triage suggests inspections based on the header. The plan shows which checks
would run and the ranges they would read; suggestions are not findings.

## Save the results

```sh
cube investigate run sample.bin --depth preview --identify --save session.json --json
cube investigate report session.json
```

`preview` inspects a short hex view and printable strings. `--identify` also
hashes the complete input so replay can check its identity. Saving requires a new
path; the record does not include the original file contents.

## Replay the checks

```sh
cube investigate replay session.json sample.bin --json
```

Replay checks the input identity and compares the reproduced results.
Changed input, unavailable parameters or incompatible contracts cause an explicit
failure. Review skipped or partial checks before treating the record as complete.

See [investigation records](../reference/topics/investigation-profiles-and-records.md)
for profiles, retained results and replay requirements.

<a id="cli"></a>
<a id="keep-an-investigation-record"></a>
<a id="python"></a>
<a id="record-an-investigation"></a>
<a id="reference"></a>
