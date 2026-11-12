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
      assert.deepEqual(execute(new Uint8Array()),new Uint8Array());
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
