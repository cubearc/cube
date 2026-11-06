import { createCube } from '@cubearc/cube';

// Public test material only. These unaudited raw-block profiles are for
// reference and legacy interoperability; they do not authenticate messages.
const cube=await createCube();
const output=cube.execute({
  operation:'cipher.block.encrypt_blocks',version:1,
  inputs:{input:new Uint8Array(8)},
  parameters:{profile:'rc2_rfc2268_v1',key_hex:'0000000000000000',effective_key_bits:63},
}).output;
if(Array.from(output,b=>b.toString(16).padStart(2,'0')).join('')!=='ebb773f993278eff')throw new Error('RC2 reference answer differs');

// Original Rijndael also defines 160-bit blocks; this is a 20-byte input,
// distinct from the fixed 128-bit block width of the AES interface.
const publicBytes = (hex: string): Uint8Array => Uint8Array.from(
  hex.match(/../g) ?? [], pair => Number.parseInt(pair, 16),
);
const wide = cube.execute({
  operation: 'cipher.block.encrypt_blocks', version: 1,
  inputs: {input: publicBytes('3243f6a8885a308d313198a2e03707344a409382')},
  parameters: {profile: 'rijndael_160_v1', key_hex: '2b7e151628aed2a6abf7158809cf4f3c'},
}).output;
if (Array.from(wide, byte => byte.toString(16).padStart(2, '0')).join('') !== '16e73aec921314c29df905432bc8968ab64b1f51') {
  throw new Error('Original Rijndael reference answer differs');
}
