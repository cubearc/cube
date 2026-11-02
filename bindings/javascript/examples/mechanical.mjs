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
