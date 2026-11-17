import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/asymmetric/ies/interoperability.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=h=>new Uint8Array(Buffer.from(h,'hex'));
const run=(operation,input,key,parameters)=>cube.execute({operation,version:1,inputs:{input,key},parameters}).output;
test('actual Wasm matches Bouncy Castle ECIES/DHIES including empty bodies and contexts',()=>{
 for(const v of vectors){
  const p=v.parameters,key=bytes(v.private_key_hex),publicKey=bytes(v.public_key_hex),plain=bytes(v.plaintext_hex),cipher=bytes(v.ciphertext_hex);
  assert.deepEqual(run('cipher.ies.decrypt',cipher,key,p),plain);
  const first=run('cipher.ies.encrypt',plain,publicKey,p);assert.notDeepEqual(first,run('cipher.ies.encrypt',plain,publicKey,p));assert.deepEqual(run('cipher.ies.decrypt',first,key,p),plain);
  const bad=cipher.slice();bad[bad.length-1]^=1;assert.throws(()=>run('cipher.ies.decrypt',bad,key,p));
  for(const field of ['derivation_hex','encoding_hex'])assert.throws(()=>run('cipher.ies.decrypt',cipher,key,{...p,[field]:p[field]+'00'}));
 }
});
