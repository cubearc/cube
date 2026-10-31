import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import init from '../dist/wasm/cube_wasm.js';

test('the real module rejects growth beyond its 256-MiB linear-memory ceiling', async () => {
  const exports = await init({ module_or_path: await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url)) });
  assert.ok(exports.memory instanceof WebAssembly.Memory);
  assert.throws(() => exports.memory.grow(4096), RangeError);
});
