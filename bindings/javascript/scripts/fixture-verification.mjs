// Qualification tooling only: check randomized cryptographic results.
// This module is never copied into the published runtime API.
import assert from 'node:assert/strict';
import {isDeepStrictEqual} from 'node:util';

export async function verifyFixtureResult({fixture,actual,contract,execute}) {
  if (fixture.verification===undefined) {
    if (!isDeepStrictEqual(actual,fixture.outputs)) throw new Error(`Actual Wasm known-answer mismatch: ${fixture.operation} (direct).`);
    return {comparison:'exact'};
  }
  assert.equal(contract.deterministic,false,'Inverse assertions cannot replace deterministic known answers.');
  const v=fixture.verification;
  const kem = kemVerification(fixture);
  if (kem) return verifyKem(fixture, actual, execute, kem);
  assert.equal(v?.kind,'inverse','Randomized fixtures require a reviewed inverse assertion.');
  assert.notEqual(v.operation,fixture.operation);
  assert.equal(Object.keys(actual).length,1,'Every randomized result port must be verified.');
  assert.ok(Object.hasOwn(actual,v.output_port));
  assert.deepEqual(v.expected_outputs,{[v.output_port]:fixture.inputs[v.input_port]},'Inverse assertion must recover the original known input.');
  let recovered;
  for (const output of [actual,fixture.outputs]) {
    const verified=await execute({operation:v.operation,version:v.operation_version,inputs:{...v.inputs,[v.input_port]:output[v.output_port]},parameters:v.parameters});
    if (output===actual) recovered=verified;
    if (!isDeepStrictEqual(verified,v.expected_outputs)) throw new Error(`Actual randomized-output verification mismatch: ${fixture.operation}.`);
  }
  return {comparison:'verified-inverse',operation:v.operation,actual:recovered,expected:v.expected_outputs};
}

export function kemVerification(fixture) {
  const kind = fixture.verification?.kind;
  if (kind !== 'kem_key_pair' && kind !== 'kem_encapsulation') return undefined;
  const keyPair = kind === 'kem_key_pair';
  const suffix = keyPair ? '.keygen' : '.encapsulate';
  assert.ok(fixture.operation.endsWith(suffix), 'KEM assertion must match its operation.');
  const family = fixture.operation.slice(0, -suffix.length);
  return {
    keyPair,
    outputs: keyPair ? ['private_key', 'public_key'] : ['ciphertext', 'shared_secret'],
    encapsulate: `${family}.encapsulate`,
    decapsulate: `${family}.decapsulate`,
  };
}

async function verifyKem(fixture, actual, execute, kem) {
  const invoke = (operation, inputs) => execute({ operation, version: 1, inputs, parameters: fixture.parameters });
  let result;
  for (const output of [actual, fixture.outputs]) {
    assert.deepEqual(Object.keys(output).sort(), kem.outputs, 'Every KEM output port must be verified.');
    const envelope = kem.keyPair ? await invoke(kem.encapsulate, { public_key: output.public_key }) : output;
    const private_key = kem.keyPair ? output.private_key : fixture.verification.private_key;
    const recovered = await invoke(kem.decapsulate, { private_key, ciphertext: envelope.ciphertext });
    const expected = { shared_secret: envelope.shared_secret };
    assert.ok(isDeepStrictEqual(recovered, expected), `KEM peers disagreed: ${fixture.operation}.`);
    if (output === actual) result = { comparison: 'verified-kem', operation: kem.decapsulate, actual: recovered, expected };
  }
  return result;
}

export function sdkWireExecutor(cube) {
  const bindings=new Map(cube.executors.map(row=>[row.operation,row]));
  return request=>{
    const binding=bindings.get(request.operation);
    assert.ok(binding,`Missing executable verifier ${request.operation}.`);
    const inputs=Object.fromEntries(Object.entries(request.inputs).map(([name,value])=>[name,
      binding.inputs[name]==='bytes'?Uint8Array.from(Buffer.from(value.base64,'base64')):
      binding.inputs[name]==='byte_array'?Uint8Array.from(value):value]));
    const outputs=cube.execute({...request,inputs});
    return Object.fromEntries(Object.entries(outputs).map(([name,value])=>[name,
      binding.outputs[name]==='bytes'?{base64:Buffer.from(value).toString('base64')}:
      binding.outputs[name]==='byte_array'?Array.from(value):value]));
  };
}
