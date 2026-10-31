import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCube } from '../dist/node.mjs';
const cube = await createCube();

test('canonical wide integers round-trip through the actual SDK without rounding or text coercion', () => {
  assert.deepEqual(cube.execute({ operation: 'number.gcd', version: 1, inputs: { a: 18446744073709551615n, b: 3n } }), { output: 3 });
  assert.throws(() => cube.execute({ operation: 'number.gcd', version: 1, inputs: { a: Number.MAX_SAFE_INTEGER + 1, b: 3 } }), { code: 'invalid_input' });
  assert.throws(() => cube.execute({ operation: 'cipher.book.decode', version: 1, inputs: { input: '1:1' }, parameters: { source: 18446744073709551615n, scheme: 'word_character', index_base: 1 } }), { code: 'invalid_parameter' });
});

test('file inspection executes arbitrary supplied bytes and does not invent OS facts', () => {
  const output = cube.execute({ operation: 'file.info.inspect', version: 2, inputs: { input: Uint8Array.of(0, 255, 128) } }).output;
  assert.deepEqual(output, { source_kind: 'artifact', size: 3, regular_file: null, modified: null, created: null, signatures: [] });
});

test('nested canonical decimal inputs retain typed behavior in actual Wasm', async () => {
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const fixture=fixtures.find(row=>row.operation==='investigation.profile.run');
  const inputs=structuredClone(fixture.inputs);
  inputs.input=Uint8Array.from(Buffer.from(inputs.input.base64,'base64'));
  inputs.profile.budgets.max_elapsed_ms='5000';
  assert.deepEqual(cube.execute({operation:fixture.operation,version:fixture.operation_version,inputs,parameters:fixture.parameters}),fixture.outputs);
});
