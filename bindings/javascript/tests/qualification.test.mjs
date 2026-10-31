import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import init, * as wasm from '../dist/wasm/cube_wasm.js';

await init({
  module_or_path: await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url)),
});

test('real Wasm preserves binary values and reports safe typed codec errors', () => {
  const input = Uint8Array.of(0, 255, 128, 1);
  assert.equal(wasm.encode_base64(input), 'AP+AAQ==');
  assert.deepEqual(wasm.decode_base64('AP+AAQ=='), input);
  assert.deepEqual(wasm.decode_base64(''), new Uint8Array());
  assert.throws(() => wasm.decode_base64('not-valid!'), (failure) => {
    const error = JSON.parse(failure);
    assert.equal(error.code, 'invalid_input');
    assert.equal(error.operation, 'encoding.base64.decode');
    assert.ok(!error.message.includes('not-valid'));
    return true;
  });
});

test('Wasm discovery retains the complete canonical ID set and exact contract wire', async () => {
  const canonicalJson = await readFile(new URL('../../../contracts/v1/catalog.json', import.meta.url), 'utf8');
  const actualJson = wasm.catalog_json();
  const canonical = JSON.parse(canonicalJson);
  const actual = JSON.parse(actualJson);
  assert.deepEqual(actual.operations.map((operation) => operation.operation), canonical.operations.map((operation) => operation.operation));
  const hash = (value) => createHash('sha256').update(value.trim()).digest('hex');
  assert.equal(hash(actualJson), hash(canonicalJson), 'Wasm contracts must match the complete canonical artifact without integer rounding');
});

test('real Core recipes preserve Unicode and captured intermediate bytes', () => {
  const definition = {
    format_version: 1,
    engine_api: 1,
    input: { source: 'inline', value: { type: 'text', data: 'Q2Fmw6k=' } },
    output: { type: 'text' },
    steps: [
      { operation: 'encoding.base64.decode', version: 1 },
      { operation: 'text.utf8.decode', version: 1 },
    ],
  };
  const result = JSON.parse(wasm.run_recipe_json(JSON.stringify(definition), '{}', Uint32Array.of(0)));
  assert.deepEqual(result.output, { type: 'text', data: 'Café' });
  assert.deepEqual(result.intermediates['0'], { type: 'bytes', base64: 'Q2Fmw6k=' });
  definition.steps[1] = { operation: 'encoding.base64.encode', version: 999 };
  assert.throws(() => wasm.run_recipe_json(JSON.stringify(definition), '{}', new Uint32Array()), (failure) => {
    const error = JSON.parse(failure);
    assert.equal(error.step, 1);
    assert.equal(error.code, 'invalid_parameter');
    return true;
  });
});

test('runtime contract identities match every reviewed fixture without JavaScript rounding', async () => {
  const fixtures = JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8'));
  const snapshot = JSON.parse(wasm.catalog_snapshot_json());
  assert.equal(snapshot.contracts.length, fixtures.length);
  for (const fixture of fixtures) {
    assert.deepEqual(snapshot.contracts.find(row => row.operation === fixture.operation), {
      operation: fixture.operation, operation_version: fixture.operation_version, contract_sha256: fixture.contract_sha256,
    }, fixture.operation);
  }
});
