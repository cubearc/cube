# Compose a pipeline or recipe

Compose compatible typed outputs and inputs. Check available adapters first; a catalog entry does not itself enable pipeline execution.

## Recipe input

Save this original fixture as `roundtrip.json`. The examples below bind its `payload` slot.

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


## CLI

### Run a pipeline or recipe

```sh
cube pipeline adapters
cube pipeline validate --input-type bytes --steps '[{"operation":"encoding.hex.encode","version":1}]'
cube pipeline run --hex 00ff --steps '[{"operation":"encoding.hex.encode","version":1}]'
# 00ff
```


```sh
cube recipe validate roundtrip.json
cube recipe run roundtrip.json --hex payload=00ff --format raw --output result.bin
cube recipe check roundtrip.json --hex payload=00ff --expect '{"type":"bytes","base64":"AP8="}'
```


```sh
cube pipeline run --text ABCDEFGHIJKLMNOPQRSTUVWXYZ --steps '[{"operation":"text.transform.masked","version":1,"parameters":{"selector":"fibonacci","exclude":{"unsigned_list":[13,21]},"index_base":1,"selected":"atbash","otherwise":"caesar","otherwise_shift":3}}]' --format json
```


```sh
cube pipeline run --text '00ff' --steps '[{"operation":"encoding.hex.decode","version":1},{"operation":"file.hex.inspect","version":1,"parameters":{"width":1}}]' --capture 0 --format json
```


## Python

### Compose a pipeline or recipe

```python
import cube

steps = [
    {"operation": "encoding.hex.encode", "version": 1},
    {"operation": "encoding.hex.decode", "version": 1},
]
plan = cube.validate_pipeline(steps, "bytes")
assert plan["output_type"] == "bytes"
result = cube.execute_pipeline(steps, b"\x00\xff", capture=[0])
assert result["output"] == b"\x00\xff"
assert result["intermediates"] == {0: "00ff"}
```


```python
from pathlib import Path
import cube

document = Path("roundtrip.json").read_text(encoding="utf-8")
assert cube.validate_recipe(document)["format_version"] == 1
assert cube.execute_recipe(document, {"payload": b"\x00\xff"})["output"] == b"\x00\xff"
assert cube.check_recipe(document, {"payload": b"\x00\xff"}, b"\x00\xff")["passed"]
```


```python
steps = [{"operation": "text.transform.masked", "version": 1, "parameters": {
    "selector": "fibonacci", "exclude": {"unsigned_list": [13, 21]},
    "index_base": 1, "selected": "atbash", "otherwise": "caesar", "otherwise_shift": 3,
    "overrides": {"text_list": ["1=雪", "1=🧊"]},
}}]
result = cube.execute_pipeline(steps, "ABCDEFGHIJKLMNOPQRSTUVWXYZ")
assert result["output"] == "🧊YXGVIJSLMNOPQRSTUVWXYZABC"
assert result["masked_audits"][0]["deterministic_text"] == "ZYXGVIJSLMNOPQRSTUVWXYZABC"
assert result["masked_audits"][0]["overrides"][1]["previous"] == "雪"
```


```python
import cube

result = cube.execute_pipeline(
    [{"operation": "encoding.hex.decode", "version": 1},
     {"operation": "file.hex.inspect", "version": 1, "parameters": {"width": 1}}],
    "00ff", capture=[0],
)
assert result["intermediates"][0] == b"\x00\xff"
assert result["output_type"] == "forensic_report"
assert result["output"]["data"]["rows"][1]["bytes"] == b"\xff"
```


Bindings are explicit `name=value` arguments. Validation checks the plan without executing it; `check` executes and compares exact typed output. A mismatch returns failure. Structured or masked-audit outputs require JSON rather than raw mode.

## Reference

- [Compose operations](../reference/topics/compose-operations.md)
