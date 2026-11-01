import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCube } from '../dist/node.mjs';
const cube = await createCube();

test('Bellaso 1553 preserves the printed table, canonical spelling and key advancement in actual Wasm', () => {
  const call = (text, parameters) => cube.execute({operation:'cipher.bellaso',version:1,inputs:{text},parameters}).text;
  const plain='LARMATATVRCHESCAPARTIRAACINQVEDILVGLIO';
  const cipher='SYBOVEYLDANVOFSZLPIINCVPNSHMLRNXOIZNRD';
  const parameters={key:'VIRTVTIOMNIAPARENT'};
  assert.equal(call(plain,parameters),cipher);
  assert.equal(call(cipher,parameters),plain);
  assert.equal(call('AA é🐈\nAA',{key:'ACD',unmapped:'preserve'}),'NT é🐈\nTN');
  assert.throws(()=>call('U',{key:'A'}),{code:'invalid_input'});
  assert.throws(()=>call('A',{key:'U'}),{code:'invalid_parameter'});
  // Native preflight rejects parameter values outside the canonical schema.
  assert.throws(()=>call('A',{key:'A'.repeat(65537)}),{code:'invalid_parameter'});
});

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

test('historical operations execute through the packaged Wasm with exact Unicode and codebook evidence', () => {
  const execute = (operation, inputs, parameters = {}) => cube.execute({ operation, version: 1, inputs, parameters });
  assert.deepEqual(execute('cipher.rot8000', { text: 'A 😀\0' }), { text: '籊 😀\0' });
  assert.deepEqual(execute('cipher.malespin', { text: 'bueno Amigo á' }), { text: 'tuani Epofi á' });
  assert.deepEqual(execute('cipher.vatsyayana', { text: 'QOEU' }, { pairs: [['Q','W'],['O','I'],['E','S'],['U','H']] }), { text: 'WISH' });
  assert.deepEqual(execute('cipher.great_cipher.encode', { units: ['Les','En','Ne','Mi','S'] }), { tokens: ['124','22','125','46','120'] });
  assert.deepEqual(execute('cipher.great_cipher.decode', { tokens: ['55','502'] }).report.units, [
    { token_index: 0, alternatives: ['I','J'] }, { token_index: 1, alternatives: ['Bien'] },
  ]);
  assert.throws(() => execute('cipher.great_cipher.decode', { tokens: ['6'] }), { code: 'invalid_input' });
  assert.throws(() => execute('cipher.vatsyayana', { text: 'A' }, { pairs: [['A','B'],['B','C']] }), { code: 'invalid_parameter' });
  assert.throws(() => execute('cipher.rot8000', { text: '\ud800' }), { code: 'invalid_input' });
  assert.throws(() => execute('cipher.nomenclator.encode', { units: Array(4050).fill('A') }, { definition: { entries: [{ alternatives: ['A'], codes: ['x'.repeat(256)] }] } }), { code: 'resource_limit' });
  const definition = { entries: [{ alternatives: ['OR'], codes: ['10'] }, { alternatives: ['D'], codes: ['05'] }], null_codes: ['99'], cancel_codes: ['27'] };
  assert.deepEqual(execute('cipher.nomenclator.decode', { tokens: ['10','05','27','99'] }, { definition }).report, {
    units: [{ token_index: 0, alternatives: ['OR'] }], null_indices: [3], canceled_indices: [1], cancel_indices: [2],
  });
});
