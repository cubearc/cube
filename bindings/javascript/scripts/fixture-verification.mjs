// Qualification tooling only: verify randomized ciphertexts through a reviewed inverse.
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
