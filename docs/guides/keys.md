# Validate offline key formats

Check the format of offline key material or derive public addresses.
These examples use public test vectors. Use them to learn the commands;
they must not be used to hold funds.

## Validate a WIF value

```sh
printf '%s' L5nJeqKmpHp4P7F8ZYyjwc5a7P4d8EabuGAzfGJk7yC1BJyzNaEd |
  cube blockchain wif --expected-network mainnet --json
```

This Bitcoin Core test vector is valid and uses the compressed mainnet form.
Validation reports format information without returning the private value.

## Derive a public address

The synthetic scalar 1 produces known public addresses:

```sh
printf '%064x' 1 | cube blockchain bitcoin --format hex --network mainnet --compressed --json
printf '%064x' 1 | cube blockchain ethereum --format hex
```

The Bitcoin address is `1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH`.
The Ethereum address is `0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf`.
Both commands run offline.

## Check a list of candidates

```sh
printf '[{"format":"hex","value":"%064x"},{"format":"wif","value":"invalid"}]' 1 |
  cube blockchain candidates --json
```

The first entry is valid; the second fails WIF validation. The report keeps each
entry's result so you can distinguish format errors.

Private export is a separate action that returns plaintext key material.
See [offline key formats](../reference/topics/offline-key-formats.md) for export
options, supported networks and handling rules.

<a id="cli"></a>
<a id="ethereum-public-derivation"></a>
<a id="ethereum-public-derivation-1"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="validate-offline-key-material"></a>
<a id="validate-offline-key-material-1"></a>
<a id="validate-supplied-key-candidates"></a>
<a id="validate-supplied-key-candidates-incrementally"></a>
