import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/asymmetric/historical_public_key/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const key=v=>new TextEncoder().encode(JSON.stringify(v));
const run=(v,action,tokens,k,parameters=v.parameters)=>cube.execute({operation:`cipher.${v.scheme}.${action}`,version:1,inputs:{tokens,key:key(k)},parameters});
test('actual Wasm executes all four exact reference schemes without numeric rounding',()=>{
 for(const v of vectors){const expected=v.candidate_tokens??v.tokens;
  assert.deepEqual(run(v,'decrypt',v.ciphertext_tokens,v.private_key)[v.scheme==='rabin'?'candidates':'tokens'],expected);
  const fresh=run(v,'encrypt',v.tokens,v.public_key).tokens;
  if(['rabin','merkle_hellman'].includes(v.scheme))assert.deepEqual(fresh,v.ciphertext_tokens);
  assert.deepEqual(run(v,'decrypt',fresh,v.private_key)[v.scheme==='rabin'?'candidates':'tokens'],expected);
  assert.deepEqual(run(v,'encrypt',[],v.public_key).tokens,[]);
  assert.throws(()=>run(v,'encrypt',v.tokens,v.public_key,{...v.parameters,max_work:1}));
  assert.throws(()=>run(v,'encrypt',v.tokens,v.public_key,{...v.parameters,max_memory_bytes:1}));
  assert.throws(()=>run(v,'encrypt',['0'.repeat(1235)],v.public_key));
 }
});
