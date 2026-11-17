import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/aes_modes/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=hex=>new Uint8Array(Buffer.from(hex,'hex'));
const run=(direction,input,parameters)=>cube.execute({operation:`cipher.aes.mode.${direction}`,version:1,inputs:{input},parameters}).output;
test('NIST AES byte modes execute in the real Wasm SDK',()=>{
 for(const v of vectors){const plain=bytes(v.plaintext_hex),encrypted=bytes(v.ciphertext_hex);assert.deepEqual(run('encrypt',plain,v.parameters),encrypted);assert.deepEqual(run('decrypt',encrypted,v.parameters),plain);}
});
test('mode settings, padding and counter exhaustion are explicit',()=>{
 const p={mode:'cbc',key_hex:'42'.repeat(16),iv_hex:'24'.repeat(16),padding:'pkcs7'};
 for(const n of [0,1,15,16,17,32]){const plain=new Uint8Array(n).fill(255);const encrypted=run('encrypt',plain,p);assert.equal(encrypted.length,(Math.floor(n/16)+1)*16);assert.deepEqual(run('decrypt',encrypted,p),plain);}
 for(const name of Object.keys(p)){const invalid={...p};delete invalid[name];assert.throws(()=>run('encrypt',new Uint8Array(),invalid),{code:'invalid_parameter'});}
 assert.throws(()=>run('decrypt',new Uint8Array(),p),{code:'invalid_input'});
 const ctr={mode:'ctr128_be',key_hex:'00'.repeat(16),iv_hex:'ff'.repeat(16)};
 assert.equal(run('encrypt',new Uint8Array(16),ctr).length,16);
 assert.throws(()=>run('encrypt',new Uint8Array(17),ctr),{code:'resource_limit'});
});
