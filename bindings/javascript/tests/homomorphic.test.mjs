import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { createCube } from '../dist/node.mjs';
import { sdkWireExecutor } from '../scripts/fixture-verification.mjs';

const fixtures = JSON.parse(await readFile(new URL(
  '../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url,
), 'utf8'));
const ids = ['cipher.bfv.add', 'cipher.bgv.add', 'cipher.ckks.add', 'cipher.tfhe.not'];

test('SEAL and TFHE encrypted results match the independent vectors in Wasm', async () => {
  const cube = await createCube();
  const execute = sdkWireExecutor(cube);
  for (const operation of ids) {
    const fixture = fixtures.find(row => row.operation === operation);
    const request = { operation, version: 1, inputs: fixture.inputs, parameters: fixture.parameters };
    const actual = execute(request);
    assert.deepEqual(actual, fixture.outputs);
    assert.throws(() => execute({ ...request, parameters: { profile: 'wrong_profile' } }));
    if (operation === 'cipher.tfhe.not') {
      assert.deepEqual(execute({ ...request, inputs: actual }), fixture.inputs);
    }
  }
});

test('Paillier and Damgård–Jurik sums match independent residues in Wasm', async () => {
  const cube = await createCube();
  const execute = sdkWireExecutor(cube);
  for (const operation of ['cipher.paillier.add', 'cipher.damgard_jurik.add']) {
    const row = fixtures.find(fixture => fixture.operation === operation);
    const request = { operation, version: 1, inputs: row.inputs, parameters: row.parameters };
    assert.deepEqual(execute(request), row.outputs);
    assert.throws(() => execute({ ...request, inputs: { ...row.inputs, public_modulus: {base64:'AA=='} } }));
  }
});

test('educational lattice cases decrypt and round-trip in Wasm', async () => {
  const cube = await createCube();
  const execute = sdkWireExecutor(cube);
  for (const family of ['ntru_encrypt', 'regev']) {
    const enc = fixtures.find(fixture => fixture.operation === `cipher.${family}.encrypt`);
    const dec = fixtures.find(fixture => fixture.operation === `cipher.${family}.decrypt`);
    const invoke = (row, inputs) => execute({ operation: row.operation, version: 1, inputs, parameters: row.parameters });
    const { output } = invoke(enc, enc.inputs);
    assert.deepEqual(invoke(dec, { ...dec.inputs, input: output }), dec.outputs);
    assert.deepEqual(invoke(dec, dec.inputs), dec.outputs);
  }
});
