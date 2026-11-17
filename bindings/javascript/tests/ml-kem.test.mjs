import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { createCube } from '../dist/node.mjs';
import { sdkWireExecutor, verifyFixtureResult } from '../scripts/fixture-verification.mjs';

const path = '../../../crates/cube-core/src/';
const vectors = JSON.parse(await readFile(new URL(`${path}asymmetric/ml_kem/vectors.json`, import.meta.url), 'utf8')).profiles;
const cube = await createCube();
const fromHex = value => Uint8Array.from(Buffer.from(value, 'hex'));
const run = (name, inputs, profile) => cube.execute({
  operation: `crypto.ml_kem.${name}`, version: 1, inputs, parameters: { profile },
});

test('all FIPS 203 parameter sets execute official and fresh cases through Wasm', () => {
  for (const { profile, decapsulation } of vectors) {
    for (const vector of decapsulation) {
      const actual = run('decapsulate', { ciphertext: fromHex(vector.c), private_key: fromHex(vector.dk) }, profile);
      assert.ok(Buffer.from(actual.shared_secret).equals(Buffer.from(vector.k, 'hex')), `ACVP case ${vector.tcId}`);
    }
    const pair = run('keygen', {}, profile);
    assert.notDeepEqual(run('keygen', {}, profile).public_key, pair.public_key);
    const envelope = run('encapsulate', { public_key: pair.public_key }, profile);
    const actual = run('decapsulate', { ciphertext: envelope.ciphertext, private_key: pair.private_key }, profile);
    assert.ok(Buffer.from(actual.shared_secret).equals(envelope.shared_secret));
    assert.throws(() => run('decapsulate', { ciphertext: envelope.ciphertext.subarray(1), private_key: pair.private_key }, profile));
  }
});

test('qualification rejects unrelated KEM keys and corrupted shared secrets', async () => {
  const fixtures = JSON.parse(await readFile(new URL(`${path}operations/contracts/conformance.json`, import.meta.url), 'utf8'));
  const execute = sdkWireExecutor(cube);
  for (const kind of ['keygen', 'encapsulate']) {
    const fixture = fixtures.find(row => row.operation === `crypto.ml_kem.${kind}`);
    const actual = await execute({ operation: fixture.operation, version: 1, inputs: fixture.inputs, parameters: fixture.parameters });
    const checked = await verifyFixtureResult({ fixture, actual, contract: { deterministic: false }, execute });
    assert.equal(checked.comparison, 'verified-kem');
    if (kind === 'keygen') {
      actual.private_key = fixture.outputs.private_key;
    } else {
      const altered = Buffer.from(actual.shared_secret.base64, 'base64');
      altered[0] ^= 1;
      actual.shared_secret.base64 = altered.toString('base64');
    }
    await assert.rejects(verifyFixtureResult({ fixture, actual, contract: { deterministic: false }, execute }), /KEM peers disagreed/);
  }
});

test('Kyber Round 3 uses the exact historical shared secret and implicit rejection', async () => {
  const vectors = JSON.parse(await readFile(new URL(`${path}asymmetric/kyber/vectors.json`, import.meta.url), 'utf8'));
  for (const vector of vectors.profiles) {
    const parameters = { profile: vector.profile };
    const private_key = fromHex(vector.dk);
    for (const [input, expected] of [['ciphertext', 'shared_secret'], ['modified_ciphertext', 'rejection_secret']]) {
      const result = cube.execute({operation:'crypto.kyber.decapsulate',version:1,inputs:{ciphertext:fromHex(vector[input]),private_key},parameters});
      assert.ok(Buffer.from(result.shared_secret).equals(fromHex(vector[expected])));
    }
  }
});

test('alternate KEM profiles match external key and ciphertext vectors in Wasm', async () => {
  const fixtures = JSON.parse(await readFile(new URL(`${path}operations/contracts/conformance.json`, import.meta.url), 'utf8'));
  const execute = sdkWireExecutor(cube);
  for (const family of ['frodo_kem', 'hqc', 'classic_mceliece', 'bike', 'ntru_kem']) {
    const operation = `crypto.${family}.decapsulate`;
    const fixture = fixtures.find(row => row.operation === operation);
    const request = { operation, version: 1, inputs: fixture.inputs, parameters: fixture.parameters };
    assert.deepEqual(execute(request), fixture.outputs);
    assert.throws(() => execute({ ...request, parameters: { profile: 'not_the_profile' } }));
  }
});
