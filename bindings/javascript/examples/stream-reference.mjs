// RFC 6229's public interoperability fixture; RC4 is broken and unauthenticated.
import { createCube } from '@cubearc/cube';
const cube = await createCube();
const parameters = { profile: 'rc4_rfc6229_v1', key_hex: '0102030405', nonce_hex: '' };
const encrypted = cube.execute({operation:'cipher.stream.encrypt',version:1,inputs:{input:new Uint8Array(16)},parameters}).output;
console.log(Buffer.from(encrypted).toString('hex'));
const restored = cube.execute({operation:'cipher.stream.decrypt',version:1,inputs:{input:encrypted},parameters}).output;
if (restored.length !== 16 || restored.some(byte => byte !== 0)) throw new Error('Reference inverse mismatch');
