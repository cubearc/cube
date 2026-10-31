import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import init, * as wasm from '../dist/wasm/cube_wasm.js';
import { pipelineCases } from './support/pipeline-cases.mjs';

await init({ module_or_path: await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url)) });
const fixtures = JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8'));

test('portable discovery reports the actual Core adapter types and versions', () => {
  const adapters = JSON.parse(wasm.pipeline_adapters_json());
  assert.equal(adapters.length, 53);
  assert.equal(new Set(adapters.map(row => row.operation)).size, adapters.length);
  const decode = adapters.find(row => row.operation === 'encoding.base64.decode');
  assert.deepEqual(decode, { operation: 'encoding.base64.decode', version: 1, input_type: 'text', output_type: 'bytes', parameters: [] });
  const xor = adapters.find(row => row.operation === 'binary.xor');
  assert.equal(xor.parameters.find(parameter => parameter.name === 'key').constraints.type, 'bytes');
});

test('every discovered adapter executes the owning known-answer fixture in real Wasm', () => {
  const cases = pipelineCases(JSON.parse(wasm.pipeline_adapters_json()), fixtures);
  for (const { operation, definition, expected } of cases) {
    const result = JSON.parse(wasm.run_recipe_json(JSON.stringify(definition), '{}', Uint32Array.of(0)));
    assert.deepEqual(result.intermediates['0'], result.output, operation);
    assert.deepEqual(result.output, expected, operation);
  }
  assert.equal(cases.length, 53);
});

test('validation rejects a later incompatible step before any execution', () => {
  const definition = {
    format_version: 1, engine_api: 1,
    input: { source: 'slot', name: 'message', type: 'text' },
    output: { type: 'text' },
    steps: [
      { operation: 'encoding.base64.decode', version: 1 },
      { operation: 'text.utf8.decode', version: 1 },
    ],
  };
  const info = JSON.parse(wasm.validate_recipe_json(JSON.stringify(definition)));
  assert.equal(info.input_slot, 'message');
  assert.equal(info.plan.steps[1].input_type, 'bytes');
  assert.ok(!JSON.stringify(info).includes('parameters'));
  definition.steps[1] = { operation: 'cipher.rot13', version: 1 };
  assert.throws(() => wasm.validate_recipe_json(JSON.stringify(definition)), failure => {
    const error = JSON.parse(failure);
    assert.equal(error.step, 1);
    assert.equal(error.code, 'invalid_parameter');
    return true;
  });
});
