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

const nemaSettings={profile:'nema_model45_operational_v1',rotors:['A','B','C','D'],drive_wheels:[12,13,14,15],initial_state:{windows:'MQQPZQSOEV'}};
const nema=cube.execute({operation:'cipher.nema.encode',version:1,inputs:{text:'A'.repeat(27),settings:nemaSettings}});
assert.equal(nema.text,'HRSBVOOTZUCRWLMGRMGVWYWOVNF');
assert.equal(nema.final_state.windows,'QPMKHPPJPU');

const typex=cube.execute({operation:'cipher.typex.encode',version:1,inputs:{text:"QWERTYUIOPASDFGHJKLXCBNMZQWERTYUIOPASDFGHJKLXCNMVA",settings:{"profile":"typex_mk22_configured_v1","wheel_set_id":"rmsk2-SP02390-interoperability","rotors":[{"wiring":"RFNVBKTIHXQGCWAEOLSMPYDZUJ","reversed":false,"id":"A"},{"wiring":"OLTGENZUJDIBYPSAFWRQMCXKHV","reversed":false,"id":"B"},{"wiring":"TWBHQDURMLNIEAKSVOYCJGXFPZ","reversed":false,"id":"C"},{"wiring":"LYUMXSFBPZONKJCEQIATGWRHDV","reversed":false,"id":"D"},{"wiring":"KGBTYSOAIVXCJPRQZNHLFWUEMD","reversed":false,"id":"E"}],"ring_settings":"AAAAA","entry_wiring":"JWULCMNOHPQZYXIRADKEGVBTSF","plugboard":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","reflector":"RYUHQSLDXPNGOKMJEAFWCZTIBV","initial_state":{"windows":"AAAAA"},"trace":true,"turnover_windows":["FMSZ","FMSZ"]}}});
assert.equal(typex.text,"HVDNGQYLGGHJOKKIOXPEQLFEMXNWIZAOMSSRMFSVVPUACYKUCN");assert.equal(typex.final_state.windows,"BIYAA");
console.log('Typex:',typex.text);

const sigaba=cube.execute({operation:'cipher.sigaba.encode',version:1,inputs:{text:"DIESZISTZEINZTOLLERZTESTZFUERZDIEZSIGABAZPUNKTZDIEZAMISZHABENZDAMALSZGLATTZMITZLEERXEICHENZVERSCHLUESSELT",settings:{"profile":"sigaba_csp889_v1","cipher_rotors":[{"id":0,"reversed":false},{"id":1,"reversed":false},{"id":2,"reversed":true},{"id":3,"reversed":false},{"id":4,"reversed":false}],"control_rotors":[{"id":5,"reversed":false},{"id":6,"reversed":false},{"id":7,"reversed":true},{"id":8,"reversed":false},{"id":9,"reversed":false}],"index_rotors":[{"id":0,"reversed":false},{"id":1,"reversed":false},{"id":2,"reversed":true},{"id":3,"reversed":false},{"id":4,"reversed":false}],"initial_state":{"cipher_windows":"LJQMJ","control_windows":"KOOON","index_windows":"09000"},"trace":true}}});
assert.equal(sigaba.text,"HHHTITJPTHJBZLQRGNCBZIKAGMFXZUXGJPWPEJPUHCSWGKBLHJKGXHHESHMXNXSZAXPIHZCCYFNZIODORLPGIDXBWRCJKPMOMIUGXSZCI");console.log('SIGABA:',sigaba.text);

const kl7=cube.execute({operation:'cipher.kl7.encode',version:1,inputs:{text:"HALLOZDIESZISTZWIEDERZEINZTESTZVVVZ",settings:{"profile":"kl7_rijmenants_v1","rotors":["A","B","C","L","D","E","F","G"],"notch_rings":[1,2,3,4,5,6,7],"notch_offsets":[14,0,0,0,0,0,0],"initial_state":{"core_positions":[34,0,1,16,1,1,1,1]},"trace":true}}});
assert.equal(kl7.text,"LPZOCRFYBRJMWHZRTSIYGTXHUODGYYIUOGP");console.log('KL7:',kl7.text);

const fialka=cube.execute({operation:'cipher.fialka.encode',version:1,inputs:{codes:[5, 18, 25, 5, 5, 18, 25, 5, 5, 18, 25, 5, 5, 18, 25, 5],settings:{"profile": "fialka_m125_6k_proton_i_v1", "rotor_order": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], "ring_offsets": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "punch_card": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29], "initial_state": {"rotor_positions": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]}, "trace": true}}});
assert.deepEqual(fialka.codes,[7, 15, 19, 0, 9, 29, 18, 26, 21, 21, 27, 8, 27, 22, 9, 1]);console.log('FIALKA:',fialka.codes);

const red=cube.execute({operation:'cipher.red.encode',version:1,inputs:{text:"JKTRYVLBZWDVJDOLVQDZ",settings:{"profile": "red_type_a_vigenere_v1", "alphabet": "LZVEYKWIHQPSOXMRDGFBUNAJCT", "pins": "11110111111111111110111111111101101111111111111", "direction": "direct", "initial_state": {"breakwheel": 38, "sixes": 0, "twenties": 10}, "trace": true}}});
assert.deepEqual(red.text,"XYXALELAKIDENXIVENIK");console.log('RED:',red.text);

const purple=cube.execute({operation:'cipher.purple.encode',version:1,inputs:{text:"FOVTATAKIDASINIMUIMINOMOXIWOIRUBESIFYXXFCKZZR",settings:{"profile": "purple_type_b_v1", "alphabet": "NOKTYUXEQLHBRMPDICJASVWGZF", "fast_switch": 2, "middle_switch": 3, "initial_state": {"switch_positions": [8, 0, 23, 5]}, "trace": true}}});
assert.deepEqual(purple.text,"ZTXODNWKCCMAVNZXYWEETUQTCIMNVEUVIWBLUAXRRTLVA");console.log('PURPLE:',purple.text);
