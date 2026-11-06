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
