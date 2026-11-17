import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/asymmetric/sm2/interoperability.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=h=>new Uint8Array(Buffer.from(h,'hex'));
const run=(operation,input,key,parameters)=>cube.execute({operation,version:1,inputs:{input,key},parameters}).output;
test('actual Wasm decrypts independent OpenSSL SM2 vectors in each exact representation',()=>{
 for(const v of vectors)for(const [profile,field] of [['sm2_sm3_c1c3c2','c1c3c2_hex'],['sm2_sm3_c1c2c3','c1c2c3_hex'],['sm2_sm3_der','der_hex']]){
  const p={profile},key=bytes(v.private_key_hex),publicKey=bytes(v.public_key_hex),plain=bytes(v.plaintext_hex),cipher=bytes(v[field]);
  assert.deepEqual(run('cipher.sm2.decrypt',cipher,key,p),plain);
  const first=run('cipher.sm2.encrypt',plain,publicKey,p);
  assert.notDeepEqual(first,run('cipher.sm2.encrypt',plain,publicKey,p));
  assert.deepEqual(run('cipher.sm2.decrypt',first,key,p),plain);
  const bad=cipher.slice();bad[bad.length-1]^=1;assert.throws(()=>run('cipher.sm2.decrypt',bad,key,p));
  assert.throws(()=>run('cipher.sm2.decrypt',cipher,new Uint8Array(32),p));
  assert.throws(()=>run('cipher.sm2.encrypt',new Uint8Array(0),publicKey,p));
 }
});
