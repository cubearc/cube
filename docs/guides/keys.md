# Validate offline key formats

Use supplied offline key material and public test vectors. Validation and public derivation are distinct from deliberate private export; keep sensitive values out of saved captures.

## CLI

### Validate offline key material

```sh
printf '%s' L5nJeqKmpHp4P7F8ZYyjwc5a7P4d8EabuGAzfGJk7yC1BJyzNaEd | cube blockchain wif --expected-network mainnet --json
printf '%s' 0000000000000000000000000000000000000000000000000000000000000001 | cube blockchain scalar --format hex --json
```


```sh
printf '%s' 0000000000000000000000000000000000000000000000000000000000000001 | cube blockchain bitcoin --format hex --network mainnet --compressed --json
cube hash ripemd160 --text 'Hello world!'
```


```sh
printf '%s' 0000000000000000000000000000000000000000000000000000000000000001 | cube blockchain export --format hex --to wif --network mainnet --compressed --private-output --stdout
```


### Ethereum public derivation

```sh
printf '%064x' 1 | cube blockchain ethereum --format hex
cube hash keccak256 --text abc
```


### Validate supplied key candidates

```sh
printf '[{"format":"hex","value":"%064x"},{"format":"wif","value":"invalid"}]' 1 |
  cube blockchain candidates --json
```


## Python

### Validate offline key material

```python
import cube

# Public Bitcoin Core test vector; validation returns no private material.
info = cube.validate_wif("L5nJeqKmpHp4P7F8ZYyjwc5a7P4d8EabuGAzfGJk7yC1BJyzNaEd", expected_network="mainnet")
assert info["valid"] and info["compressed"]
assert info["network_family"] == "mainnet"
assert cube.validate_private_scalar(bytes(31) + b"\x01")["valid"]
```


```python
import cube

# Synthetic public scalar-1 vector.
public = cube.derive_bitcoin_address(bytes(31) + b"\x01", network="mainnet", compressed=True)
assert public["address"] == "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH"
assert isinstance(public["public_key"], bytes) and len(public["public_key"]) == 33
assert cube.ripemd160(b"Hello world!") == "7f772647d88750add82d8e1a7a3e5c0902a346a3"
```


```python
import cube

# Public synthetic test vector. This call deliberately returns plaintext private data.
key = bytes(31) + b"\x01"
private_text = cube.export_private_key(key, output="wif", network="mainnet", compressed=True)
assert cube.export_private_key(private_text, format="wif", output="raw") == key
assert cube.export_private_key(private_text, format="wif", output="hex") == key.hex()
```


### Ethereum public derivation

```python
import cube

eth = cube.derive_ethereum_address(bytes(31) + b"\x01")  # public test scalar
assert eth["address"] == "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf"
assert len(eth["public_key_xy"]) == 64
assert len(eth["address_bytes"]) == 20
assert cube.keccak256(b"abc") == "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"
```


### Validate supplied key candidates incrementally

```python
import cube

batch = cube.validate_key_candidates([
    {"format": "hex", "value": f"{1:064x}"},  # public test scalar
    {"format": "wif", "value": "invalid"},
])
assert batch.plan()["entries"] == 2
first = next(batch)
assert first["index"] == 0 and first["valid"]
assert not batch.progress()["complete"]
assert [entry["valid"] for entry in batch] == [False]
assert batch.progress()["complete"]
```


Examples use public synthetic vectors. An explicit private-return method can return plaintext material; Python values and explicit file writes remain the caller's responsibility. Do not use real secrets to learn the interface.

## Reference

- [Offline key formats](../reference/topics/offline-key-formats.md)
