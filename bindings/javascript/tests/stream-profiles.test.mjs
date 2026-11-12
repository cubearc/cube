import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCube } from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/stream/reference/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=hex=>Uint8Array.from(Buffer.from(hex,'hex'));
test('stream references execute independent answers in actual Wasm',()=>{
  const seen=new Set();
  for(const v of vectors){
    seen.add(v.parameters.profile);
    for(const [mode,input,expected] of [['encrypt',v.plaintext_hex,v.ciphertext_hex],['decrypt',v.ciphertext_hex,v.plaintext_hex]]){
      const execute=input=>cube.execute({operation:`cipher.stream.${mode}`,version:1,inputs:{input},parameters:v.parameters}).output;
      assert.deepEqual(execute(bytes(input)),bytes(expected),v.parameters.profile);
      const empty={...v.parameters};if('data_bits'in empty)empty.data_bits=0;
      assert.deepEqual(cube.execute({operation:`cipher.stream.${mode}`,version:1,inputs:{input:new Uint8Array()},parameters:empty}).output,new Uint8Array());
      for(const change of [{key_hex:''},{key_hex:'00'.repeat(257)},{nonce_hex:null}]){
        assert.throws(()=>cube.execute({operation:`cipher.stream.${mode}`,version:1,inputs:{input:new Uint8Array()},parameters:{...v.parameters,...change}}),{code:'invalid_parameter'});
      }
    }
  }
  const contract=JSON.parse(cube.catalogJson()).operations.find(o=>o.operation==='cipher.stream.encrypt');
  assert.deepEqual(seen,new Set(contract.parameters.oneOf.map(p=>p.properties.profile.const)));
});
test('stream settings and byte limits fail without private excerpts',()=>{
  for(const mode of ['encrypt','decrypt']){
    const invoke=(input,parameters)=>cube.execute({operation:`cipher.stream.${mode}`,version:1,inputs:{input},parameters});
    const good={profile:'rc4_rfc6229_v1',key_hex:'0102030405',nonce_hex:''};
    for(const change of [{key_hex:'SECRET'},{nonce_hex:null},{discard_bytes:0},{profile:'rc4_drop_v1'},{profile:'rc4_drop_v1',discard_bytes:null}]){
      assert.throws(()=>invoke(new Uint8Array(),{...good,...change}),error=>error.code==='invalid_parameter'&&!error.message.includes('SECRET'));
    }
    assert.throws(()=>invoke(new Uint8Array(65537),good),{code:'resource_limit'});
  }
});

test('SEAL exposes the full last position and rejects counter wrap',()=>{
 for(const profile of ['seal3_be_l32768_v1','seal3_le_l32768_v1'])for(const mode of ['encrypt','decrypt']){
  const parameters={profile,key_hex:'00'.repeat(20),nonce_hex:'ffffffff'};
  const invoke=n=>cube.execute({operation:`cipher.stream.${mode}`,version:1,inputs:{input:new Uint8Array(n)},parameters}).output;
  assert.equal(invoke(4096).length,4096);
  assert.throws(()=>invoke(4097),{code:'resource_limit'});
 }
});

test('packed bit profiles preserve padding and reject mismatched lengths',()=>{
 const seen=new Set();
 for(const v of vectors){if(!('data_bits'in v.parameters)||seen.has(v.parameters.profile))continue;seen.add(v.parameters.profile);
  const parameters={...v.parameters,data_bits:1};const plain=new Uint8Array([parameters.profile.startsWith('e0_')?1:128]);
  const invoke=(mode,input,p=parameters)=>cube.execute({operation:`cipher.stream.${mode}`,version:1,inputs:{input},parameters:p}).output;
  assert.deepEqual(invoke('decrypt',invoke('encrypt',plain)),plain);
  for(const mode of ['encrypt','decrypt'])for(const input of [new Uint8Array(),new Uint8Array([129]),new Uint8Array(2)])assert.throws(()=>invoke(mode,input),{code:'invalid_parameter'});
  for(const data_bits of [null,524289])assert.throws(()=>invoke('encrypt',new Uint8Array(),{...parameters,data_bits}),{code:'invalid_parameter'});
 }
 assert.equal(seen.size,7);
});
