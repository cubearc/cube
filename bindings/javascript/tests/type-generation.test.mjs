import assert from 'node:assert/strict';
import test from 'node:test';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {generateDirectTypes} from '../scripts/generate-types.mjs';

test('conditional profile constraints preserve required TypeScript parameters',async()=>{
 const root=await mkdtemp(join(tmpdir(),'cube-profile-types-'));
 try {
  const parameters={type:'object',additionalProperties:false,properties:{profile:{enum:['exact']},key_hex:{type:'string'},nonce_hex:{type:'string'}},required:['profile','key_hex','nonce_hex'],allOf:[{if:{properties:{profile:{const:'exact'}}},then:{properties:{nonce_hex:{pattern:'^[0-9a-f]{24}$'}}}}]};
  await generateDirectTypes(join(root,'direct.d.ts'),[{operation:'example',version:1,inputs:{},outputs:{}}],{operations:[{operation:'example',inputs:[],outputs:[],parameters}]});
  await writeFile(join(root,'index.d.ts'),"export type JsonValue=null|boolean|number|string|readonly JsonValue[]|{readonly [key:string]:JsonValue};\n");
  await writeFile(join(root,'usage.mts'),`import type {DirectParameters} from './direct.js';
const valid:DirectParameters['example']={profile:'exact',key_hex:'00',nonce_hex:'00'};
// @ts-expect-error a conditional schema must retain the required nonce
const missingNonce:DirectParameters['example']={profile:'exact',key_hex:'00'};
// @ts-expect-error an unsupported profile is rejected statically
const wrongProfile:DirectParameters['example']={profile:'guess',key_hex:'00',nonce_hex:'00'};
void valid;void missingNonce;void wrongProfile;
`);
  const compiler=fileURLToPath(new URL('../node_modules/typescript/bin/tsc',import.meta.url));
  const result=spawnSync(process.execPath,[compiler,'--noEmit','--strict','--target','ES2022','--module','NodeNext','--moduleResolution','NodeNext',join(root,'usage.mts')],{encoding:'utf8'});
  assert.equal(result.status,0,result.stdout+result.stderr);
 } finally {await rm(root,{recursive:true,force:true});}
});
