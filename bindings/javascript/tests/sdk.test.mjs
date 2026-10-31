import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCube, CubeError } from '../dist/node.mjs';

const cube = await createCube();

test('the public API preserves every registered direct executor fixture and all output ports', async () => {
  const fixtures = JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8'));
  for (const executor of cube.executors) {
    const fixture = fixtures.find(row => row.operation === executor.operation);
    const inputs = Object.fromEntries(Object.entries(fixture.inputs).map(([name, value]) => [name,
      executor.inputs[name] === 'bytes' ? Uint8Array.from(Buffer.from(value.base64, 'base64'))
        : executor.inputs[name] === 'byte_array' ? Uint8Array.from(value) : value,
    ]));
    const expected = Object.fromEntries(Object.entries(fixture.outputs).map(([name, value]) => [name,
      executor.outputs[name] === 'bytes' ? Uint8Array.from(Buffer.from(value.base64, 'base64'))
        : executor.outputs[name] === 'byte_array' ? Uint8Array.from(value) : value,
    ]));
    assert.deepEqual(cube.execute({ operation: executor.operation, version: executor.version, inputs, parameters: fixture.parameters }), expected, executor.operation);
  }
});

test('public Node API shares initialization and preserves exact named binary results', async () => {
  assert.equal(await createCube(), cube);
  assert.equal(await createCube({ wasm: new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url) }), cube);
  const original = Buffer.from([0, 255, 128, 1]);
  const encoded = cube.execute({ operation: 'encoding.base64.encode', version: 1, inputs: { input: original } });
  assert.deepEqual(encoded, { output: 'AP+AAQ==' });
  const decoded = cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: encoded.output } });
  assert.deepEqual(decoded, { output: Uint8Array.of(0, 255, 128, 1) });
  decoded.output.fill(0);
  assert.deepEqual(original, Buffer.from([0, 255, 128, 1]));
  assert.throws(() => cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: '!', extra: 'private' } }), { code: 'invalid_input' });
  assert.throws(() => cube.execute({ operation: 'encoding.base64.decode', version: 2, inputs: { input: '' } }), { code: 'incompatible_version' });
  assert.throws(() => cube.execute({ operation: 'not.a.cube.operation', version: 1, inputs: {} }), { code: 'unsupported_operation' });
});

test('public recipe API uses Core preflight and preserves bytes and exact integer transport', () => {
  const recipe = {
    format_version: 1, engine_api: 1,
    input: { source: 'slot', name: 'message', type: 'text' },
    output: { type: 'text' },
    steps: [
      { operation: 'encoding.base64.decode', version: 1 },
      { operation: 'text.utf8.decode', version: 1 },
    ],
  };
  assert.equal(cube.validateRecipe(recipe).plan.steps.length, 2);
  const result = cube.runRecipe(recipe, { message: { type: 'text', data: 'Q2Fmw6k=' } }, { captures: [0] });
  assert.deepEqual(result.output, { type: 'text', data: 'Café' });
  assert.deepEqual(result.intermediates['0'], { type: 'bytes', data: new TextEncoder().encode('Café') });
  const identity = {
    format_version: 1, engine_api: 1,
    input: { source: 'inline', value: { type: 'byte_range', offset: 9007199254740993n, length: null } },
    output: { type: 'byte_range' }, steps: [],
  };
  assert.equal(cube.runRecipe(identity).output.offset, '9007199254740993');
  assert.equal(cube.runRecipe(cube.recipeJson(identity)).output.offset, '9007199254740993');
  identity.input.value.offset = Number.MAX_SAFE_INTEGER + 1;
  assert.throws(() => cube.runRecipe(identity), { code: 'invalid_input' });
  assert.throws(() => cube.runRecipe(recipe, {}, { captures: [2 ** 32] }), { code: 'invalid_parameter' });
});

test('typed errors reject malformed Unicode and protect private inputs', () => {
  for (const value of ['private-secret!', '\ud800']) {
    assert.throws(() => cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: value } }), error => {
      assert.ok(error instanceof CubeError);
      assert.equal(error.code, 'invalid_input');
      if (value === 'private-secret!') assert.equal(error.operation, 'encoding.base64.decode');
      assert.ok(!error.message.includes('private-secret'));
      return true;
    });
  }
});
