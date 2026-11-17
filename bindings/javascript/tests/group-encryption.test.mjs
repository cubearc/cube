import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/asymmetric/group_encryption/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=h=>new Uint8Array(Buffer.from(h,'hex'));
const run=(scheme,action,input,key,parameters)=>cube.execute({operation:`cipher.${scheme}.${action}`,version:1,inputs:{input,key},parameters}).output;
test('actual Wasm matches independent raw group vectors and verifies fresh ciphertexts',()=>{
 for(const v of vectors){const p=v.parameters,key=bytes(v.private_key_hex),publicKey=bytes(v.public_key_hex),plain=bytes(v.plaintext_hex),cipher=bytes(v.ciphertext_hex);
  assert.deepEqual(run(v.scheme,'decrypt',cipher,key,p),plain);
  const first=run(v.scheme,'encrypt',plain,publicKey,p);assert.notDeepEqual(first,run(v.scheme,'encrypt',plain,publicKey,p));assert.deepEqual(run(v.scheme,'decrypt',first,key,p),plain);
  assert.throws(()=>run(v.scheme,'decrypt',cipher,new Uint8Array(key.length),p));assert.throws(()=>run(v.scheme,'encrypt',new Uint8Array(256),publicKey,p));
  if(v.scheme==='cramer_shoup'){const bad=cipher.slice();bad[bad.length-1]^=1;assert.throws(()=>run(v.scheme,'decrypt',bad,key,p));assert.throws(()=>run(v.scheme,'decrypt',cipher,key,{...p,label_hex:p.label_hex+'00'}));}
 }
});
