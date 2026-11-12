import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/stream/reference/authentication/vectors.json',import.meta.url),'utf8'));
const bytes=hex=>new Uint8Array(Buffer.from(hex,'hex'));
const cube=await createCube();
const encrypt=(input,parameters)=>cube.execute({operation:'cipher.stream.auth.encrypt',version:1,inputs:{input},parameters});
const decrypt=(input,tag,parameters)=>cube.execute({operation:'cipher.stream.auth.decrypt',version:1,inputs:{input,tag},parameters}).output;
test('original Grain128a independent ciphertext/tag answers execute in actual Wasm',()=>{
 for(const v of vectors){const plain=bytes(v.plaintext_hex),ciphertext=bytes(v.ciphertext_hex),tag=bytes(v.tag_hex);
  assert.deepEqual(encrypt(plain,v.parameters),{output:ciphertext,tag});
  assert.deepEqual(decrypt(ciphertext,tag,v.parameters),plain);
  const forged=tag.slice();forged[0]^=128;
  assert.throws(()=>decrypt(ciphertext,forged,v.parameters),{code:'invalid_input'});
 }
});
test('authenticated input, detached tag and private settings fail explicitly',()=>{
 const v=vectors[4],parameters=v.parameters,plain=bytes(v.plaintext_hex),ciphertext=bytes(v.ciphertext_hex),tag=bytes(v.tag_hex);
 for(const name of Object.keys(parameters)){const missing={...parameters};delete missing[name];assert.throws(()=>encrypt(plain,missing),{code:'invalid_parameter'});}
 for(const change of [{key_hex:'SECRET'},{nonce_hex:null},{data_bits:524289},{tag_bits:0},{tag_bits:33},{tag_bits:1.5},{associated_data:'SECRET'}])assert.throws(()=>encrypt(plain,{...parameters,...change}),error=>error.code==='invalid_parameter'&&!error.message.includes('SECRET'));
 const bad=ciphertext.slice();bad[0]^=128;assert.throws(()=>decrypt(bad,tag,parameters),{code:'invalid_input'});
 const padding=plain.slice();padding[padding.length-1]|=1;assert.throws(()=>encrypt(padding,parameters),{code:'invalid_input'});
 for(const invalid of [new Uint8Array(),new Uint8Array(5)])assert.throws(()=>decrypt(ciphertext,invalid,parameters),{code:'invalid_input'});
 assert.throws(()=>encrypt(new Uint8Array(65537),{...parameters,data_bits:524288}),{code:'resource_limit'});
});
