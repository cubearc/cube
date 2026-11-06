import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCube } from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/block_profiles/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const bytes=hex=>Uint8Array.from(Buffer.from(hex,'hex'));
test('all block reference profiles execute independent vectors in actual Wasm',()=>{
  const seen=new Set();
  for(const vector of vectors){
    const parameters={profile:vector.profile,key_hex:vector.key_hex};
    if(vector.effective_key_bits!==undefined)parameters.effective_key_bits=vector.effective_key_bits;
    if(vector.tweak_hex!==undefined)parameters.tweak_hex=vector.tweak_hex;
    seen.add(vector.profile);
    for(const [mode,input,expected] of [['encrypt_blocks',vector.plaintext_hex,vector.ciphertext_hex],['decrypt_blocks',vector.ciphertext_hex,vector.plaintext_hex]]){
      const execute=input=>cube.execute({operation:`cipher.block.${mode}`,version:1,inputs:{input},parameters}).output;
      assert.deepEqual(execute(bytes(input)),bytes(expected),`${vector.profile} ${mode}`);
      assert.deepEqual(execute(bytes(input.repeat(3))),bytes(expected.repeat(3)));
      assert.deepEqual(execute(new Uint8Array()),new Uint8Array());
    }
  }
  const contract=JSON.parse(cube.catalogJson()).operations.find(o=>o.operation==='cipher.block.encrypt_blocks');
  assert.deepEqual(seen,new Set(contract.parameters.oneOf.map(p=>p.properties.profile.const)));
});
test('profile settings and partial blocks fail without echoing secrets',()=>{
  for(const mode of ['encrypt_blocks','decrypt_blocks']){
    const execute=(input,parameters)=>cube.execute({operation:`cipher.block.${mode}`,version:1,inputs:{input},parameters});
    for(const parameters of [
      {profile:'des_fips46_v1',key_hex:'SECRET'},
      {profile:'craft_2020_v1',key_hex:'00'.repeat(16)},
      {profile:'craft_2020_v1',key_hex:'00'.repeat(16),tweak_hex:'SECRET'},
      {profile:'craft_2020_v1',key_hex:'00'.repeat(16),tweak_hex:null},
      {profile:'idea_v1',key_hex:'00'.repeat(16),tweak_hex:'00'.repeat(8)},
      {profile:'des_fips46_v1',key_hex:'00'.repeat(8),effective_key_bits:64},
      {profile:'rc2_rfc2268_v1',key_hex:'88'},
      {profile:'rc2_rfc2268_v1',key_hex:'88',effective_key_bits:null},
      {profile:'rc2_rfc2268_v1',key_hex:'88',effective_key_bits:true},
      {profile:'rc2_rfc2268_v1',key_hex:'88',effective_key_bits:1025},
    ])assert.throws(()=>execute(new Uint8Array(),parameters),error=>error.code==='invalid_parameter'&&!error.message.includes('SECRET'));
    assert.throws(()=>execute(new Uint8Array(7),{profile:'idea_v1',key_hex:'00'.repeat(16)}),{code:'invalid_input'});
    assert.throws(()=>execute(new Uint8Array(1048577),{profile:'idea_v1',key_hex:'00'.repeat(16)}),{code:'resource_limit'});
  }
});


const xxteaVectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/xxtea/vectors.json',import.meta.url),'utf8'));
test('XXTEA executes complete word sequences, with no string or length wrapper',()=>{
  for(const v of xxteaVectors){
    for(const [mode,input,expected] of [['encrypt_words',v.plaintext_hex,v.ciphertext_hex],['decrypt_words',v.ciphertext_hex,v.plaintext_hex]]){
      const execute=input=>cube.execute({operation:`cipher.xxtea.${mode}`,version:1,inputs:{input},parameters:{key_hex:v.key_hex}}).output;
      assert.deepEqual(execute(bytes(input)),bytes(expected));
      assert.deepEqual(execute(new Uint8Array()),new Uint8Array());
    }
  }
  for(const mode of ['encrypt_words','decrypt_words']){
    const execute=(input,key_hex='00'.repeat(16))=>cube.execute({operation:`cipher.xxtea.${mode}`,version:1,inputs:{input},parameters:{key_hex}});
    assert.throws(()=>execute(new Uint8Array(),'SECRET'),e=>e.code==='invalid_parameter'&&!e.message.includes('SECRET'));
    for(const n of [1,4,7,9])assert.throws(()=>execute(new Uint8Array(n)),{code:'invalid_input'});
    assert.throws(()=>execute(new Uint8Array(65537)),{code:'resource_limit'});
  }
  const key_hex='00'.repeat(16),data=new Uint8Array(65536).fill(0x95);
  const encrypted=cube.execute({operation:'cipher.xxtea.encrypt_words',version:1,inputs:{input:data},parameters:{key_hex}}).output;
  assert.deepEqual(cube.execute({operation:'cipher.xxtea.decrypt_words',version:1,inputs:{input:encrypted},parameters:{key_hex}}).output,data);
});
