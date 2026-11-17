import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createCube} from '../dist/node.mjs';
const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/fpe/vectors.json',import.meta.url),'utf8'));
const cube=await createCube();
const run=(direction,input,parameters)=>cube.execute({operation:`cipher.fpe.${direction}`,version:1,inputs:{input},parameters}).output;
test('NIST and FF3-1 independent numeral strings execute through Wasm',()=>{
 for(const v of vectors){assert.deepEqual(run('encrypt',v.plaintext,v.parameters),v.ciphertext);assert.deepEqual(run('decrypt',v.ciphertext,v.parameters),v.plaintext);}
});
test('FPE domain, digits and bounded private settings are enforced',()=>{
 const p={profile:'ff1_sp800_38g_integer',radix:10,key_hex:'00'.repeat(16),tweak_hex:''};
 for(const digits of [[],Array(5).fill(0),[0,0,0,0,0,10],Array(4097).fill(0)])assert.throws(()=>run('encrypt',digits,p));
 assert.throws(()=>run('encrypt',Array(6).fill(0),{...p,key_hex:'SECRET'}),e=>!e.message.includes('SECRET'));
});
