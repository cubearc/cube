import assert from 'node:assert/strict';
import {createCube} from '@cubearc/cube';
const cube=await createCube();
const settings={profile:'m94_v1',order:Array.from({length:25},(_,i)=>i+1),row_offsets:[1]};
const encoded=cube.execute({operation:'cipher.m94.encode',version:1,inputs:{text:'AAAAA',settings}});
assert.equal(encoded.text,'BCDEF');
assert.equal(cube.execute({operation:'cipher.m94.decode',version:1,inputs:{text:encoded.text,settings}}).text,'AAAAA');
console.log('M-94:',encoded.text);
