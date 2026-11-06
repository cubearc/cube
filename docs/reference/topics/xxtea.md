# XXTEA word sequences

`cipher.xxtea.encrypt_words` and `cipher.xxtea.decrypt_words` implement Corrected
Block TEA over one complete sequence of little-endian 32-bit words. They are
low-level educational interoperability operations without authentication.
Decryption with a wrong valid key produces bytes without checking integrity.

Supply exactly sixteen raw key bytes. Input is empty, or contains at least two
whole words: 8, 12, 16, ... bytes, up to 64 KiB. A single four-byte word is rejected.
The entire input is one variable-width block, using `6 + floor(52 / word_count)`
cycles. Splitting a sixteen-byte input into two eight-byte calls changes the
result. There is no padding, text encoding, key normalization or length suffix.

The Rust APIs are `cube_core::ciphers::xxtea::{Key, encrypt, decrypt}`. Construct
the validated key with `Key::new(&bytes)`. The native Python SDK exports
`xxtea_encrypt_words(data, key)` and `xxtea_decrypt_words(data, key)`:

```python
import cube

# Public qualification material only.
key = bytes(16)
ciphertext = cube.xxtea_encrypt_words(bytes(8), key)
assert ciphertext.hex() == "ab043705808c5d57"
assert cube.xxtea_decrypt_words(ciphertext, key) == bytes(8)
```

JavaScript, TypeScript and WebAssembly use the same Core operations:

```typescript
import { createCube } from '@cubearc/cube';

const cube = await createCube();
const ciphertext = cube.execute({
  operation: 'cipher.xxtea.encrypt_words', version: 1,
  inputs: { input: new Uint8Array(8) },
  parameters: { key_hex: '00'.repeat(16) },
}).output;
```

The CLI reads a private raw key file and exact binary input. It writes exact
binary output; omit `--file` to read stdin:

```sh
cube cipher xxtea encrypt --key-file key.bin --file words.bin > encrypted.bin
cube cipher xxtea decrypt --key-file key.bin --file encrypted.bin > restored.bin
```

Core validates keys and alignment before word expansion, enforces the 64 KiB
input/output ceiling, and limits private parameter JSON to 4096 bytes. Keys and
owned word buffers are wiped on drop; caller, parser, SDK and OS copies are
outside that guarantee. Errors omit private values. Private inputs and outputs
require explicit retention in native Play. These direct operations have no
pipeline adapter or automatic capture permission.

Fourteen independent raw-kernel answers cover both directions, nonzero keys
and multiple word counts. CLI, installed native Python, packaged Wasm and native
Play consume the same corpus. See the [backend qualification
record](../../contributing/block-backends.md), [canonical
contracts](../../../contracts/v1/catalog.json), and the
[block-profile reference](block-profiles.md) for TEA and XTEA's distinct fixed
block profiles.
