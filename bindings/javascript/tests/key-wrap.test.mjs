import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const root=new URL('../../../crates/cube-core/src/ciphers/key_wrap/',import.meta.url);
const valid=JSON.parse(await readFile(new URL('vectors.json',root),'utf8'));
const invalid=JSON.parse(await readFile(new URL('invalid_vectors.json',root),'utf8'));
const cube=await createCube();const bytes=h=>new Uint8Array(Buffer.from(h,'hex'));
const run=(direction,input,parameters)=>cube.execute({operation:`cipher.aes.key_wrap.${direction}`,version:1,inputs:{input},parameters}).output;
test('NIST KW/KWP profiles execute in actual Wasm with all KEK sizes',()=>{
 for(const v of valid){const plain=bytes(v.plaintext_hex),wrapped=bytes(v.wrapped_hex);assert.deepEqual(run('wrap',plain,v.parameters),wrapped);assert.deepEqual(run('unwrap',wrapped,v.parameters),plain);}
});
test('NIST adversarial cases release no unverified key material',()=>{
 for(const v of invalid)assert.throws(()=>run('unwrap',bytes(v.wrapped_hex),v.parameters),{code:'invalid_input'});
});
test('key lengths, private settings and maximum data bounds remain exact',()=>{
 const p={profile:'aes_kwp_rfc5649',key_hex:'42'.repeat(16)};
 for(const n of [1,7,8,9,16,65536]){const data=new Uint8Array(n).fill(36),wrapped=run('wrap',data,p);assert.equal(wrapped.length,Math.ceil(n/8)*8+8);assert.deepEqual(run('unwrap',wrapped,p),data);}
 assert.throws(()=>run('wrap',new Uint8Array(),p),{code:'invalid_input'});
 assert.throws(()=>run('wrap',new Uint8Array(65537),p),{code:'resource_limit'});
 assert.throws(()=>run('wrap',new Uint8Array(1),{profile:p.profile,key_hex:'SECRET'}),e=>e.code==='invalid_parameter'&&!e.message.includes('SECRET'));
});
