import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/asymmetric/ecdh/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=h=>new Uint8Array(Buffer.from(h,'hex'));
const run=(input,parameters)=>cube.execute({operation:'crypto.ecdh.agree',version:1,inputs:{input},parameters}).output;
test('RFC7748 and RFC5903 exact ECDH shared secrets execute in actual Wasm',()=>{
  for(const v of vectors)assert.deepEqual(run(bytes(v.peer_hex),v.parameters),bytes(v.shared_hex));
});
test('weak points, wrong keys and bounds fail without secret diagnostics',()=>{
  for(const v of vectors)for(const n of [0,1,31,32,33,65,66])assert.throws(()=>run(new Uint8Array(n),v.parameters));
  assert.throws(()=>run(new Uint8Array(32),{profile:'x25519_rfc7748',private_key_hex:'SECRET'}),e=>e.code==='invalid_parameter'&&!e.message.includes('SECRET'));
});
