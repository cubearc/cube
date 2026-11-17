import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {webcrypto} from 'node:crypto';
import {createCube} from '../dist/node.mjs';
import {verifyFixtureResult,sdkWireExecutor} from '../scripts/fixture-verification.mjs';
const root=new URL('../../../crates/cube-core/src/asymmetric/rsa/',import.meta.url);
const load=async file=>JSON.parse(await readFile(new URL(file,root),'utf8'));
const interop=await load('interoperability.json');const oaep=(await load('wycheproof_oaep.json')).testGroups[0];const legacy=(await load('implicit_rejection_vectors.json')).groups;
const cube=await createCube();const bytes=h=>new Uint8Array(Buffer.from(h,'hex'));
const run=(operation,data,key,parameters)=>cube.execute({operation,version:1,inputs:{input:data,key},parameters}).output;
const encrypt=(data,key,p)=>run('cipher.rsa.oaep.encrypt',data,key,p);const decrypt=(data,key,p)=>run('cipher.rsa.oaep.decrypt',data,key,p);
test('actual Wasm ciphertexts decrypt independently through native WebCrypto',async()=>{
 for(const v of interop){
  const plain=bytes(v.plaintext_hex);assert.deepEqual(decrypt(bytes(v.ciphertext_hex),bytes(v.private_key_hex),v.private_parameters),plain);
  const a=encrypt(plain,bytes(v.public_key_hex),v.public_parameters),b=encrypt(plain,bytes(v.public_key_hex),v.public_parameters);assert.notDeepEqual(a,b);assert.equal(a.length,Math.ceil(v.bits/8));
  let key=Buffer.from(v.private_key_hex,'hex');if(v.private_parameters.key_format==='pkcs8_pem')key=Buffer.from(key.toString().replace(/-----[^-]+-----|\s/g,''),'base64');
  const native=await webcrypto.subtle.importKey('pkcs8',key,{name:'RSA-OAEP',hash:'SHA-256'},false,['decrypt']);
  const recovered=await webcrypto.subtle.decrypt({name:'RSA-OAEP',label:bytes(v.private_parameters.label_hex)},native,a);assert.deepEqual(new Uint8Array(recovered),plain);
 }
});
test('Wycheproof OAEP outcomes retain the explicitly reviewed CFRG10 exception',()=>{
 for(const v of oaep.tests){const p={profile:'rsa_oaep_sha256_mgf1_sha256_cfrg10',key_format:'pkcs8_der',label_hex:v.label};
  if(v.result==='valid'||v.tcId===23)assert.deepEqual(decrypt(bytes(v.ct),bytes(oaep.privateKeyPkcs8),p),bytes(v.msg));else assert.throws(()=>decrypt(bytes(v.ct),bytes(oaep.privateKeyPkcs8),p),{code:'invalid_input'});
 }
});
test('legacy implicit rejection returns the independent synthetic bytes without a validity signal',()=>{
 for(const group of legacy)for(const v of group.cases){const f=()=>run('cipher.rsa.pkcs1_v1_5.decrypt',bytes(v.ciphertext_hex),bytes(group.key_hex),group.parameters);
  if(v.expect==='public_error')assert.throws(f,{code:'invalid_input'});else assert.deepEqual(f(),bytes(v.output_hex));
 }
});
test('randomized conformance rejects forged ciphertexts, wrong plaintext and missing assertions',async()=>{
 const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
 const fixture=fixtures.find(v=>v.operation==='cipher.rsa.oaep.encrypt');const contract=JSON.parse(cube.catalogJson()).operations.find(v=>v.operation===fixture.operation);const execute=sdkWireExecutor(cube);
 const actual=execute({operation:fixture.operation,version:1,inputs:fixture.inputs,parameters:fixture.parameters});
 await verifyFixtureResult({fixture,actual,contract,execute});
 const bad=structuredClone(actual);const raw=Buffer.from(bad.output.base64,'base64');raw[raw.length-1]^=1;bad.output.base64=raw.toString('base64');
 await assert.rejects(verifyFixtureResult({fixture,actual:bad,contract,execute}));
 const wrong=structuredClone(fixture);wrong.verification.expected_outputs.output.base64='AA==';await assert.rejects(verifyFixtureResult({fixture:wrong,actual,contract,execute}));
 const missing=structuredClone(fixture);delete missing.verification;await assert.rejects(verifyFixtureResult({fixture:missing,actual,contract,execute}));
 await assert.rejects(verifyFixtureResult({fixture,actual,contract:{...contract,deterministic:true},execute}));
});
