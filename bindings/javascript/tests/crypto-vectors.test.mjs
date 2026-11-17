import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/modern_aead/wycheproof_vectors.json',import.meta.url),'utf8'));
const cube=await createCube();const bytes=s=>new Uint8Array(Buffer.from(s,'hex'));
test('Wycheproof AES-GCM valid and adversarial cases execute in actual Wasm',()=>{for(const v of vectors){const call=()=>cube.execute({operation:'cipher.aead.decrypt',version:1,inputs:{input:bytes(v.ciphertext_hex),aad:bytes(v.aad_hex),tag:bytes(v.tag_hex)},parameters:v.parameters});if(v.valid)assert.deepEqual(call().output,bytes(v.plaintext_hex));else assert.throws(call,{code:'invalid_input'});}});
