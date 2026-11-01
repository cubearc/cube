import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCube } from '../dist/node.mjs';
const cube = await createCube();
test('numeric profiles execute exact fixtures and bounded multiset recovery in Wasm', async () => {
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const cases=fixtures.filter(r=>/^cipher\.(prime_number|prime_multiplication|modulo|triliteral)\./.test(r.operation));
  assert.equal(cases.length,8);
  for(const r of cases) assert.deepEqual(cube.execute({operation:r.operation,version:1,inputs:r.inputs,parameters:r.parameters}),r.outputs,r.operation);
  const parameters={profile:'prime_multiset_v1',index:{alphabet:'AB',first_index:1,max_prime:3},max_factor_checks:13};
  const encoded=cube.execute({operation:'cipher.prime_multiplication.encode',version:1,inputs:{groups:['AB','BA','']},parameters});
  assert.deepEqual(encoded.groups[0],encoded.groups[1]);
  assert.equal(encoded.groups[2].product,'1');
  const decoded=cube.execute({operation:'cipher.prime_multiplication.decode',version:1,inputs:{products:['6','6','1','8']},parameters});
  assert.equal(decoded.factor_checks,13);
  assert.throws(()=>cube.execute({operation:'cipher.prime_multiplication.decode',version:1,inputs:{products:['6','6','1','8']},parameters:{...parameters,max_factor_checks:12}}),{code:'resource_limit'});
  for(const token of ['0','01','5','6.0','340282366920938463463374607431768211456']) assert.throws(()=>cube.execute({operation:'cipher.prime_multiplication.decode',version:1,inputs:{products:[token]},parameters}),{code:'invalid_input'});
});

test('finite Gajewski Solresol retains source synonyms, accents, aliases and positions', async () => {
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const cases=fixtures.filter(r=>r.operation.startsWith('symbol.solresol.'));
  assert.equal(cases.length,2);
  for(const r of cases) assert.deepEqual(cube.execute({operation:r.operation,version:1,inputs:r.inputs,parameters:r.parameters}),r.outputs,r.operation);
  const p={profile:'gajewski_1902_monosyllables_v1'};
  const encoded=cube.execute({operation:'symbol.solresol.encode',version:1,inputs:{terms:['non','oui','à','si (condition)']},parameters:p});
  assert.deepEqual(encoded.tokens,['do','si','fa','sol']);
  const decoded=cube.execute({operation:'symbol.solresol.decode',version:1,inputs:{tokens:['d','si','r','ré','do','do']},parameters:p});
  assert.deepEqual(decoded.readings[0].alternatives,['non','pas','point','ne','ni']);
  assert.deepEqual(decoded.tokens,['d','si','r','ré','do','do']);
  assert.equal(decoded.readings[5].token_index,5);
  assert.equal(decoded.provenance.definition_sha256,encoded.provenance.definition_sha256);
  for(const token of ['re','re\u0301','rr','do-ré','C4']) assert.throws(()=>cube.execute({operation:'symbol.solresol.decode',version:1,inputs:{tokens:[token]},parameters:p}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'symbol.solresol.decode',version:1,inputs:{tokens:[]},parameters:{}}),{code:'invalid_parameter'});
});

test('Key Phrase alternatives and paginated whole words execute through the actual SDK', async () => {
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const cases=fixtures.filter(row=>/^cipher\.(key_phrase|arnold)\./.test(row.operation));
  assert.equal(cases.length,4);
  for (const row of cases) assert.deepEqual(cube.execute({operation:row.operation,version:1,inputs:row.inputs,parameters:row.parameters}),row.outputs,row.operation);
  const p={phrase:'GIVEMELIBERTYORGIVEMEDEATH'};
  const result=cube.execute({operation:'cipher.key_phrase.decode',version:1,inputs:{text:'I'},parameters:p});
  assert.equal(result.text,null);
  assert.deepEqual(result.records[0].alternatives,['B','H','Q']);
  assert.equal(cube.execute({operation:'cipher.key_phrase.encode',version:1,inputs:{text:'bhq *'},parameters:p}).text,'III *');
  assert.throws(()=>cube.execute({operation:'cipher.key_phrase.decode',version:1,inputs:{text:'Z'},parameters:p}),{code:'invalid_input'});
  const ap={profile:'arnold_paged_words_v1',source_id:'synthetic:edition-one',index_base:0,choice:'round_robin'};
  const pages=['A B\nA 猫.','A'];
  const encoded=cube.execute({operation:'cipher.arnold.encode',version:1,inputs:{pages,words:['A','A','A','A']},parameters:ap});
  assert.deepEqual(encoded.coordinates,[{page:0,line:0,word:0},{page:0,line:1,word:0},{page:1,line:0,word:0},{page:0,line:0,word:0}]);
  assert.deepEqual(cube.execute({operation:'cipher.arnold.decode',version:1,inputs:{pages,coordinates:encoded.coordinates},parameters:ap}),encoded);
  assert.throws(()=>cube.execute({operation:'cipher.arnold.decode',version:1,inputs:{pages,coordinates:[{page:0,line:9,word:0}]},parameters:ap}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'cipher.arnold.encode',version:1,inputs:{pages:[1],words:[]},parameters:ap}),{code:'invalid_input'});
});

test('ACA constructors preserve repeated positions and execute all five Headlines answers', async () => {
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const cases=fixtures.filter(row=>/^cipher\.(numbered_key|headlines)\./.test(row.operation));
  assert.equal(cases.length,4);
  for (const fixture of cases) assert.deepEqual(cube.execute({operation:fixture.operation,version:1,inputs:fixture.inputs,parameters:fixture.parameters}),fixture.outputs,fixture.operation);
  const p={alphabet:'abcdefghijklmnopqrstuvwxyz',key:'I like ciphers.',preparation:'ascii_lowercase_letters',rotation:18,origin:'0',width:2,choice:'round_robin'};
  assert.deepEqual(cube.execute({operation:'cipher.numbered_key.encode',version:1,inputs:{text:'A ee iii'},parameters:p}).tokens,['23','15','20','11','13','17']);
  assert.deepEqual(cube.execute({operation:'cipher.numbered_key.encode',version:1,inputs:{text:'eee'},parameters:{...p,choice:'first'}}).tokens,['15','15','15']);
  assert.equal(cube.execute({operation:'cipher.numbered_key.decode',version:1,inputs:{tokens:['20']},parameters:{...p,choice:'first'}}).text,'e');
  assert.throws(()=>cube.execute({operation:'cipher.numbered_key.decode',version:1,inputs:{tokens:['1']},parameters:p}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'cipher.numbered_key.encode',version:1,inputs:{text:'é'},parameters:p}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'cipher.numbered_key.encode',version:1,inputs:{text:'a'},parameters:{...p,origin:'340282366920938463463374607431768211455'}}),{code:'invalid_parameter'});
  const hp={profile:'aca_base',hat:'APOTHECARY',key:'CHEMIST',setting:'DRUGS',punctuation:'preserve'};
  assert.deepEqual(cube.execute({operation:'cipher.headlines.encode',version:1,inputs:{headlines:['','','','','']},parameters:hp}).headlines,['','','','','']);
  assert.throws(()=>cube.execute({operation:'cipher.headlines.encode',version:1,inputs:{headlines:['','','','','']},parameters:{...hp,setting:'CRUGS'}}),{code:'invalid_parameter'});
});

test('qualified table and VIC profiles execute every published answer through the actual SDK', async () => {
  const names=new Set(['cadenus','swagman','nihilist_transposition','ubchi','cm_bifid','vic']);
  let checked=0;
  for (const domain of ['transposition','fractionation']) {
    const vectors=JSON.parse(await readFile(new URL(`../../../crates/cube-core/src/ciphers/${domain}/vectors.json`,import.meta.url),'utf8'));
    for (const v of vectors.filter(v=>names.has(v.operation.split('.')[1]))) {
      assert.deepEqual(cube.execute({operation:v.operation,version:1,inputs:v.inputs,parameters:v.parameters}),v.outputs,v.operation);
      checked++;
    }
  }
  assert.equal(checked,12);
  const text='X🐈e\u0301\0Z';
  const encoded=cube.execute({operation:'cipher.cadenus.encode',version:1,inputs:{text},parameters:{keyword:'EASY',padding:'X'}});
  assert.equal(cube.execute({operation:'cipher.cadenus.decode',version:1,inputs:{text:encoded.text},parameters:{keyword:'EASY',padding:'X',original_length:encoded.original_length}}).text,text);
  assert.throws(()=>cube.execute({operation:'cipher.swagman.encode',version:1,inputs:{text:''},parameters:{key:[[1,2,3,4],[1,2,3,4],[1,2,3,4],[1,2,3,4]]}}),{code:'invalid_parameter'});
  const p={phrase:'ALLTHEPEOPLEAREDEADB',mnemonic:'ASINTOER',date_digits:'391752',personal_number:15,message_indicator:'60115',split_at:0,padding:'0'};
  const cipher=cube.execute({operation:'cipher.vic.encode',version:1,inputs:{text:'A..'},parameters:p}).text;
  assert.equal(cube.execute({operation:'cipher.vic.decode',version:1,inputs:{text:cipher},parameters:p}).text,'A..');
  assert.throws(()=>cube.execute({operation:'cipher.vic.encode',version:1,inputs:{text:'123'},parameters:p}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'cipher.vic.decode',version:1,inputs:{text:'00000'},parameters:p}),{code:'invalid_input'});
});

test('Bellaso 1553 preserves the printed table, canonical spelling and key advancement in actual Wasm', () => {
  const call = (text, parameters) => cube.execute({operation:'cipher.bellaso',version:1,inputs:{text},parameters}).text;
  const plain='LARMATATVRCHESCAPARTIRAACINQVEDILVGLIO';
  const cipher='SYBOVEYLDANVOFSZLPIINCVPNSHMLRNXOIZNRD';
  const parameters={key:'VIRTVTIOMNIAPARENT'};
  assert.equal(call(plain,parameters),cipher);
  assert.equal(call(cipher,parameters),plain);
  assert.equal(call('AA é🐈\nAA',{key:'ACD',unmapped:'preserve'}),'NT é🐈\nTN');
  assert.throws(()=>call('U',{key:'A'}),{code:'invalid_input'});
  assert.throws(()=>call('A',{key:'U'}),{code:'invalid_parameter'});
  // Native preflight rejects parameter values outside the canonical schema.
  assert.throws(()=>call('A',{key:'A'.repeat(65537)}),{code:'invalid_parameter'});
});

test('canonical wide integers round-trip through the actual SDK without rounding or text coercion', () => {
  assert.deepEqual(cube.execute({ operation: 'number.gcd', version: 1, inputs: { a: 18446744073709551615n, b: 3n } }), { output: 3 });
  assert.throws(() => cube.execute({ operation: 'number.gcd', version: 1, inputs: { a: Number.MAX_SAFE_INTEGER + 1, b: 3 } }), { code: 'invalid_input' });
  assert.throws(() => cube.execute({ operation: 'cipher.book.decode', version: 1, inputs: { input: '1:1' }, parameters: { source: 18446744073709551615n, scheme: 'word_character', index_base: 1 } }), { code: 'invalid_parameter' });
});

test('file inspection executes arbitrary supplied bytes and does not invent OS facts', () => {
  const output = cube.execute({ operation: 'file.info.inspect', version: 2, inputs: { input: Uint8Array.of(0, 255, 128) } }).output;
  assert.deepEqual(output, { source_kind: 'artifact', size: 3, regular_file: null, modified: null, created: null, signatures: [] });
});

test('nested canonical decimal inputs retain typed behavior in actual Wasm', async () => {
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const fixture=fixtures.find(row=>row.operation==='investigation.profile.run');
  const inputs=structuredClone(fixture.inputs);
  inputs.input=Uint8Array.from(Buffer.from(inputs.input.base64,'base64'));
  inputs.profile.budgets.max_elapsed_ms='5000';
  assert.deepEqual(cube.execute({operation:fixture.operation,version:fixture.operation_version,inputs,parameters:fixture.parameters}),fixture.outputs);
});

test('historical operations execute through the packaged Wasm with exact Unicode and codebook evidence', () => {
  const execute = (operation, inputs, parameters = {}) => cube.execute({ operation, version: 1, inputs, parameters });
  assert.deepEqual(execute('cipher.rot8000', { text: 'A 😀\0' }), { text: '籊 😀\0' });
  assert.deepEqual(execute('cipher.malespin', { text: 'bueno Amigo á' }), { text: 'tuani Epofi á' });
  assert.deepEqual(execute('cipher.vatsyayana', { text: 'QOEU' }, { pairs: [['Q','W'],['O','I'],['E','S'],['U','H']] }), { text: 'WISH' });
  assert.deepEqual(execute('cipher.great_cipher.encode', { units: ['Les','En','Ne','Mi','S'] }), { tokens: ['124','22','125','46','120'] });
  assert.deepEqual(execute('cipher.great_cipher.decode', { tokens: ['55','502'] }).report.units, [
    { token_index: 0, alternatives: ['I','J'] }, { token_index: 1, alternatives: ['Bien'] },
  ]);
  assert.throws(() => execute('cipher.great_cipher.decode', { tokens: ['6'] }), { code: 'invalid_input' });
  assert.throws(() => execute('cipher.vatsyayana', { text: 'A' }, { pairs: [['A','B'],['B','C']] }), { code: 'invalid_parameter' });
  assert.throws(() => execute('cipher.rot8000', { text: '\ud800' }), { code: 'invalid_input' });
  assert.throws(() => execute('cipher.nomenclator.encode', { units: Array(4050).fill('A') }, { definition: { entries: [{ alternatives: ['A'], codes: ['x'.repeat(256)] }] } }), { code: 'resource_limit' });
  const definition = { entries: [{ alternatives: ['OR'], codes: ['10'] }, { alternatives: ['D'], codes: ['05'] }], null_codes: ['99'], cancel_codes: ['27'] };
  assert.deepEqual(execute('cipher.nomenclator.decode', { tokens: ['10','05','27','99'] }, { definition }).report, {
    units: [{ token_index: 0, alternatives: ['OR'] }], null_indices: [3], canceled_indices: [1], cancel_indices: [2],
  });
});

test('dated symbolic alphabets execute every published assignment in actual Wasm', async () => {
  const rows=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/symbols/alphabet/vectors.json',import.meta.url),'utf8'));
  for(const row of rows) {
    const parameters={profile:row.profile};
    const encoded=cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:row.units},parameters});
    assert.deepEqual(encoded.tokens,row.tokens);
    const decoded=cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:row.tokens},parameters});
    assert.deepEqual(decoded.readings,row.units.map((unit,token_index)=>({token_index,alternatives:[unit]})));
    assert.deepEqual(decoded.tokens,row.tokens);
    assert.deepEqual(decoded.provenance,encoded.provenance);
    assert.deepEqual(cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[]},parameters}).readings,[]);
  }
  const parameters={profile:'aurebesh_ucsur_2016_v1'};
  for(const unit of ['a','Ch','CHAT','0','é','']) assert.throws(()=>cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:[unit]},parameters}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:['\ue8d0']},parameters}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:[]},parameters:{}}),{code:'invalid_parameter'});
  assert.throws(()=>cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:Array(65537).fill('A')},parameters}),{code:'resource_limit'});
});

test('all source-permitted geometric orientations and bird aliases execute without guessing', async () => {
  const aliases=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/symbols/alphabet/aliases.json',import.meta.url),'utf8'));
  assert.equal(aliases.length,147);
  for(const row of aliases) {
    const r=cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[row.token]},parameters:{profile:row.profile}});
    assert.deepEqual(r.tokens,[row.token]);
    assert.deepEqual(r.readings,[{token_index:0,alternatives:row.alternatives}]);
  }
  for(const [profile,token] of [['pigpen_haverhill_pairs_v1','grid:ES:cross'],['tic_tac_toe_dcode_v1','grid:ES:dot'],['templars_bothwell_gosse_v1','angle:north:cross'],['betamaze_cliff_2006_v1','.../.../...'],['clock_quarter_hour_zero_v1','12:00'],['birds_nrtech_four_features_v1','B27']]) assert.throws(()=>cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[token]},parameters:{profile}}),{code:'invalid_input'});
});
