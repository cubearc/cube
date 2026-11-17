import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/asymmetric/dh/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=h=>new Uint8Array(Buffer.from(h,'hex'));
const run=(input,key,parameters)=>cube.execute({operation:'crypto.dh.agree',version:1,inputs:{input,key},parameters}).output;
test('all five RFC7919 named groups execute exact independent answers in actual Wasm',()=>{
 for(const v of vectors)assert.deepEqual(run(bytes(v.peer_hex),bytes(v.private_key_hex),v.parameters),bytes(v.shared_hex));
});
test('DH malformed peers, zero keys and unknown groups fail',()=>{
 const v=vectors[0];for(const n of [0,1,255,256,257,1025])assert.throws(()=>run(new Uint8Array(n),bytes(v.private_key_hex),v.parameters));
 assert.throws(()=>run(bytes(v.peer_hex),new Uint8Array(32),v.parameters));
 assert.throws(()=>run(bytes(v.peer_hex),bytes(v.private_key_hex),{profile:'custom'}));
});
