import assert from 'node:assert/strict';
import {createCube} from '@cubearc/cube';
const cube=await createCube();
const settings={profile:'m94_v1',order:Array.from({length:25},(_,i)=>i+1),row_offsets:[1]};
const encoded=cube.execute({operation:'cipher.m94.encode',version:1,inputs:{text:'AAAAA',settings}});
assert.equal(encoded.text,'BCDEF');
assert.equal(cube.execute({operation:'cipher.m94.decode',version:1,inputs:{text:encoded.text,settings}}).text,'AAAAA');
console.log('M-94:',encoded.text);

const wheatstone={profile:'wheatstone_1879_v1',cipher_alphabet:'FBKSYRDLTZAGMUNHOVCIPWEJQX',initial_state:{plain_position:0,cipher_position:0}};
const clock=cube.execute({operation:'cipher.wheatstone.encode',version:1,inputs:{text:'THE+QUARTER+',settings:wheatstone}});
assert.equal(clock.text,'PZLSPQREQAJD');
assert.deepEqual(clock.final_state,{plain_position:0,cipher_position:6});
console.log('Wheatstone:',clock.text);
const alberti={profile:'alberti_di_palma_2009_v1',initial_index:'c'};
assert.equal(cube.execute({operation:'cipher.alberti.decode',version:1,inputs:{text:'sfzefqqfxkmohihuy',settings:alberti}}).text,'SILVI4ALOVES2CATS');
const kryha={profile:'kryha_callimahos_1973_v1',plain_alphabet:'ABCDEFGHIJKLMNOPQRSTUVWXYZ',cipher_alphabet:'HYDRAULICBEFGJKMNOPQSTVWXZ',initial_state:{rotation:0,selector:1}};
assert.equal(cube.execute({operation:'cipher.kryha.encode',version:1,inputs:{text:'A'.repeat(17),settings:kryha}}).text,'HIJSZUGPHLNTYLJQX');

const lorenzSettings={profile:'lorenz_sz40_v1',chi:[41,31,29,26,23].map(n=>'0'.repeat(n)),psi:[43,47,51,53,59].map(n=>'0'.repeat(n)),mu61:'0'.repeat(61),mu37:'0'.repeat(37),plaintext_limitation:false,initial_state:{chi:[0,0,0,0,0],psi:[0,0,0,0,0],mu61:0,mu37:0,chi2_previous:false,psi1_previous:false,plain5_previous:[false,false]}};
// Explicit all-dot qualification key: the five-bit values remain unchanged.
assert.deepEqual(cube.execute({operation:'cipher.lorenz.encode',version:1,inputs:{codes:[0,31,5],settings:lorenzSettings}}).codes,[0,31,5]);

// Deliberately all-inactive educational M-209 key, not a historical key sheet.
const hagelinSettings={profile:'m209_standard_v1',pins:[26,25,23,21,19,17].map(n=>'0'.repeat(n)),lugs:Array.from({length:27},()=>Array(6).fill(false)),initial_state:[0,0,0,0,0,0]};
const hagelin=cube.execute({operation:'cipher.hagelin.encode',version:1,inputs:{text:'ABC',settings:hagelinSettings}});
assert.equal(hagelin.text,'ZYX');
assert.deepEqual(hagelin.final_state,[3,3,3,3,3,3]);

// Synthetic all-active cam facts; callers identify their actual fixed cam set.
const t52abSettings={profile:'siemens_t52ab_configured_v1',wheel_set_id:'synthetic-all-active',cam_patterns:[47,53,59,61,64,65,67,69,71,73].map(n=>'1'.repeat(n)),wheel_order:[0,1,2,3,4,5,6,7,8,9],socket_pairs:[[1,3],[2,10],[4,5],[6,7],[8,9]],initial_state:Array(10).fill(0)};
const t52ab=cube.execute({operation:'cipher.t52ab.encode',version:1,inputs:{codes:[0,31,5],settings:t52abSettings}});
assert.deepEqual(t52ab.codes,[31,0,26]);
assert.deepEqual(t52ab.final_state,Array(10).fill(3));
