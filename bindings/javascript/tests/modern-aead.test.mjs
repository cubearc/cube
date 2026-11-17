import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const root=new URL('../../../crates/cube-core/src/ciphers/modern_aead/',import.meta.url);
const vectors=JSON.parse(await readFile(new URL('vectors.json',root),'utf8'));
const sivVectors=JSON.parse(await readFile(new URL('siv_vectors.json',root),'utf8'));
const cube=await createCube();
const bytes=hex=>new Uint8Array(Buffer.from(hex,'hex'));
const call=(operation,inputs,parameters)=>cube.execute({operation,version:1,inputs,parameters});
test('all exact AEAD profiles execute independent ciphertext/tag answers in actual Wasm',()=>{
 for(const v of vectors){const input=bytes(v.plaintext_hex),aad=bytes(v.aad_hex),output=bytes(v.ciphertext_hex),tag=bytes(v.tag_hex),parameters=v.parameters;
  assert.deepEqual(call('cipher.aead.encrypt',{input,aad},parameters),{output,tag},parameters.profile);
  assert.deepEqual(call('cipher.aead.decrypt',{input:output,aad,tag},parameters).output,input);
  const forged=tag.slice();forged[0]^=1;assert.throws(()=>call('cipher.aead.decrypt',{input:output,aad,tag:forged},parameters),{code:'invalid_input'});
 }
});
test('nonce-free SIV authenticates ordered AAD components and detached tags',()=>{
 for(const v of sivVectors){const input=bytes(v.plaintext_hex),output=bytes(v.ciphertext_hex),tag=bytes(v.tag_hex),aad_components=v.aad_components_hex.map(hex=>Array.from(bytes(hex))),parameters=v.parameters;
  assert.deepEqual(call('cipher.aes.siv.encrypt',{input,aad_components},parameters),{output,tag});
  assert.deepEqual(call('cipher.aes.siv.decrypt',{input:output,aad_components,tag},parameters).output,input);
  assert.throws(()=>call('cipher.aes.siv.decrypt',{input:output,aad_components:[...aad_components,[]],tag},parameters),{code:'invalid_input'});
 }
 const parameters={profile:'aes128_siv_cmac_nonce_free',key_hex:'42'.repeat(32)};
 const none=call('cipher.aes.siv.encrypt',{input:new Uint8Array(),aad_components:[]},parameters);
 const empty=call('cipher.aes.siv.encrypt',{input:new Uint8Array(),aad_components:[[]]},parameters);
 assert.notDeepEqual(none.tag,empty.tag);
 assert.throws(()=>call('cipher.aes.siv.encrypt',{input:new Uint8Array(),aad_components:Array.from({length:127},()=>[])},parameters),{code:'invalid_input'});
});
test('required key, nonce, AAD, tag and exact profiles cannot be inferred',()=>{
 const v=vectors[0],parameters=v.parameters,input=new Uint8Array(),aad=new Uint8Array();
 for(const name of Object.keys(parameters)){const p={...parameters};delete p[name];assert.throws(()=>call('cipher.aead.encrypt',{input,aad},p),{code:'invalid_parameter'});}
 assert.throws(()=>call('cipher.aead.encrypt',{input},parameters),{code:'invalid_input'});
 assert.throws(()=>call('cipher.aead.decrypt',{input,aad},parameters),{code:'invalid_input'});
 assert.throws(()=>call('cipher.aead.encrypt',{input,aad},{...parameters,key_hex:'SECRET'}),e=>e.code==='invalid_parameter'&&!e.message.includes('SECRET'));
 assert.throws(()=>call('cipher.aead.encrypt',{input:new Uint8Array(16*1024*1024+1),aad},parameters),{code:'resource_limit'});
});
