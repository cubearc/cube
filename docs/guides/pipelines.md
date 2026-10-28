# Compose a pipeline or recipe

Chain operations when each output type matches the next input.
Start by checking which operations your build can run in a pipeline:

```sh
cube pipeline adapters
```

Some operations are available only as direct commands or SDK calls.

## Run a pipeline

This pipeline encodes two bytes as hexadecimal text:

```sh
cube pipeline validate --input-type bytes --steps '[{"operation":"encoding.hex.encode","version":1}]'
cube pipeline run --hex 00ff --steps '[{"operation":"encoding.hex.encode","version":1}]'
# 00ff
```

Validation checks the steps and types without running them. Execution returns
the text `00ff`.

## Save a reusable recipe

Save this as `roundtrip.json`. It encodes bytes as hexadecimal, then decodes them
back to bytes:

```json
{
  "format_version": 1,
  "engine_api": 1,
  "input": {"source": "slot", "name": "payload", "type": "bytes"},
  "output": {"type": "bytes"},
  "steps": [
    {"operation": "encoding.hex.encode", "version": 1},
    {"operation": "encoding.hex.decode", "version": 1}
  ]
}
```

Bind the `payload` input, run the recipe, and check its expected result:

```sh
cube recipe validate roundtrip.json
cube recipe run roundtrip.json --hex payload=00ff --format raw --output result.bin
cube recipe check roundtrip.json --hex payload=00ff --expect '{"type":"bytes","base64":"AP8="}'
```

`result.bin` contains the two original bytes. The check passes because `AP8=`
encodes those same bytes. A different expected value makes the check fail.

## Keep an intermediate result

```sh
cube pipeline run --text '00ff' --steps '[{"operation":"encoding.hex.decode","version":1},{"operation":"file.hex.inspect","version":1,"parameters":{"width":1}}]' --capture 0 --format json
```

The report includes the decoded bytes from step 0 and the final hex inspection.
Use JSON for structured reports and masked-transformation audits; raw mode cannot
represent them.

See [pipeline types, recipes and limits](../reference/topics/compose-operations.md)
for compatibility and parameter rules.

<a id="cli"></a>
<a id="compose-a-pipeline-or-recipe-1"></a>
<a id="python"></a>
<a id="recipe-input"></a>
<a id="reference"></a>
<a id="run-a-pipeline-or-recipe"></a>
