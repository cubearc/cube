// Original SKEW2011 Grain128a fixture, with detached32-bit plaintext MAC.
import assert from 'node:assert/strict';
import {createCube} from '@cubearc/cube';
const cube=await createCube();
const parameters={profile:'grain128a_skew2011_msb_v1',key_hex:'00'.repeat(16),nonce_hex:'00'.repeat(12),data_bits:41,tag_bits:32};
const input=Uint8Array.from([0x12,0x34,0x56,0x78,0x9e,0x80]);
const {output,tag}=cube.execute({operation:'cipher.stream.auth.encrypt',version:1,inputs:{input},parameters});
assert.equal(Buffer.from(tag).toString('hex'),'4c8dcab1');
assert.deepEqual(cube.execute({operation:'cipher.stream.auth.decrypt',version:1,inputs:{input:output,tag},parameters}).output,input);
const forged=tag.slice();forged[0]^=128;
assert.throws(()=>cube.execute({operation:'cipher.stream.auth.decrypt',version:1,inputs:{input:output,tag:forged},parameters}),{code:'invalid_input'});
console.log(JSON.stringify({ciphertext_hex:Buffer.from(output).toString('hex'),tag_hex:Buffer.from(tag).toString('hex')}));
