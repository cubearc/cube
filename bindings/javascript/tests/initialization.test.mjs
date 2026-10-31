import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { createCube } from '../dist/browser.mjs';

test('a mismatched valid Wasm module is rejected before instantiation and initialization can retry', async () => {
  const emptyModule = Uint8Array.of(0, 97, 115, 109, 1, 0, 0, 0);
  await assert.rejects(createCube({ wasm: emptyModule }), { code: 'incompatible_runtime' });
  const bytes = await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url));
  const first = createCube({ wasm: bytes });
  const second = createCube({ wasm: bytes });
  bytes.fill(0); // Initialization must already own a frozen copy.
  assert.equal(first, second);
  const cube = await first;
  assert.equal(cube.execute({ operation: 'encoding.base64.encode', version: 1, inputs: { input: Uint8Array.of(255) } }).output, '/w==');
  assert.equal((await second), cube);
  await assert.rejects(createCube({ wasm: emptyModule }), { code: 'initialization_conflict' });
  assert.match(cube.build.wasmSha256, /^[0-9a-f]{64}$/);
  assert.equal(cube.build.sdkVersion, '0.18.0');
});
