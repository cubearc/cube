import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCube } from '../dist/node.mjs';
const cube = await createCube();
const documentNodeCount=value=>1+(value&&typeof value==='object'?Object.values(value).reduce((sum,v)=>sum+documentNodeCount(v),0):0);
test('historical wheel and strip profiles execute through actual Wasm with explicit rows', async()=>{
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const rows=fixtures.filter(r=>/^cipher\.(m94|jefferson|m138a)\./.test(r.operation));
  assert.equal(rows.length,6);
  for(const r of rows)assert.deepEqual(cube.execute({operation:r.operation,version:1,inputs:r.inputs,parameters:r.parameters}),r.outputs,r.operation);
  const settings=rows.find(r=>r.operation==='cipher.m94.encode').inputs.settings;
  assert.throws(()=>cube.execute({operation:'cipher.m94.encode',version:1,inputs:{text:'SECRET',settings:{...settings,row_offsets:[]}}}),{code:'invalid_parameter'});
  assert.throws(()=>cube.execute({operation:'cipher.m94.encode',version:1,inputs:{text:'a',settings:{...settings,row_offsets:[1]}}}),{code:'invalid_input'});
  assert.throws(()=>cube.execute({operation:'cipher.m94.encode',version:1,inputs:{text:'A'.repeat(1048577),settings:{...settings,row_offsets:Array(41944).fill(1)}}}),{code:'resource_limit'});
});
test('stateful historical mechanisms preserve published answers, continuation and bounded traces in Wasm', async () => {
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const rows=fixtures.filter(r=>/^cipher\.(wheatstone|kryha|alberti)\./.test(r.operation));
  assert.equal(rows.length,6);
  for(const r of rows) {
    assert.deepEqual(cube.execute({operation:r.operation,version:1,inputs:r.inputs,parameters:{}}),r.outputs,r.operation);
    const settings={...r.inputs.settings,trace:false};
    const call=(text,settings)=>cube.execute({operation:r.operation,version:1,inputs:{text,settings}});
    const first=call(r.inputs.text.slice(0,3),settings);
    const key=r.operation.includes('.alberti.')?'index':'state';
    const second=call(r.inputs.text.slice(3),{...settings,[`initial_${key}`]:first[`final_${key}`]});
    assert.equal(first.text+second.text,r.outputs.text);
    assert.deepEqual(second[`final_${key}`],r.outputs[`final_${key}`]);
    assert.throws(()=>call('秘密',settings),{code:'invalid_input'});
    assert.throws(()=>call('A'.repeat(65537),{...settings,trace:true}),{code:'resource_limit'});
  }
});

test('Lorenz five-bit models execute their source pins, delayed KT history and continuation in actual Wasm', async () => {
  const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/lorenz_vectors.json',import.meta.url),'utf8'));
  const execute=(direction,codes,settings)=>cube.execute({operation:`cipher.lorenz.${direction}`,version:1,inputs:{codes,settings}});
  assert.deepEqual(execute('encode',v.codes,v.settings).codes,v.expected);
  for(const profile of ['lorenz_sz40_v1','lorenz_sz42a_v1','lorenz_sz42b_v1']) {
    for(const plaintext_limitation of [false,true]) {
      if(profile==='lorenz_sz40_v1'&&plaintext_limitation)continue;
      const settings={...v.settings,profile,plaintext_limitation,trace:false};
      const codes=Array.from({length:32},(_,i)=>i);
      const whole=execute('encode',codes,settings);
      const decoded=execute('decode',whole.codes,settings);
      assert.deepEqual(decoded.codes,codes);
      assert.deepEqual(decoded.final_state,whole.final_state);
      const first=execute('encode',codes.slice(0,7),settings);
      const second=execute('encode',codes.slice(7),{...settings,initial_state:first.final_state});
      assert.deepEqual([...first.codes,...second.codes],whole.codes);
      assert.deepEqual(second.final_state,whole.final_state);
    }
  }
  for(const codes of [[32],[-1],[true]])assert.throws(()=>execute('encode',codes,v.settings),{code:'invalid_input'});
  const settings={...v.settings,trace:false};
  const empty={operation:'cipher.lorenz.encode',version:1,inputs:{codes:[],settings},parameters:{}};
  const maxCodes=cube.limits.direct.documentNodes-documentNodeCount(empty);
  assert.equal(execute('encode',Array(maxCodes).fill(31),settings).codes.length,maxCodes);
  assert.throws(()=>execute('encode',Array(maxCodes+1).fill(31),settings),{code:'resource_limit'});
  assert.throws(()=>execute('encode',Array(1048576).fill(31),settings),{code:'resource_limit'});
  assert.throws(()=>execute('encode',Array(1048577).fill(0),{...v.settings,trace:false}),{code:'resource_limit'});
  assert.throws(()=>execute('encode',Array(65537).fill(0),v.settings),{code:'resource_limit'});
  assert.throws(()=>execute('encode',[],{...v.settings,plaintext_limitation:true}),{code:'invalid_parameter'});
});

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
    assert.deepEqual(decoded.readings,row.units.map((unit,token_index)=>({token_index,alternatives:row.alternatives?.[token_index]??[unit]})));
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

test('all source-permitted aliases execute with exact forms and ambiguity', async () => {
  const aliases=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/symbols/alphabet/aliases.json',import.meta.url),'utf8'));
  assert.equal(aliases.length,187);
  for(const row of aliases) {
    const r=cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[row.token]},parameters:{profile:row.profile}});
    assert.deepEqual(r.tokens,[row.token]);
    assert.deepEqual(r.readings,[{token_index:0,alternatives:row.alternatives}]);
  }
  for(const [profile,token] of [['pigpen_haverhill_pairs_v1','grid:ES:cross'],['tic_tac_toe_dcode_v1','grid:ES:dot'],['templars_bothwell_gosse_v1','angle:north:cross'],['betamaze_cliff_2006_v1','.../.../...'],['clock_quarter_hour_zero_v1','12:00'],['birds_nrtech_four_features_v1','B27']]) assert.throws(()=>cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[token]},parameters:{profile}}),{code:'invalid_input'});
});

test('complete symbolic reports enforce output budgets including provenance', () => {
  const parameters={profile:'birds_nrtech_four_features_v1'};
  // The SDK's stricter transport-node budget rejects this before Core invocation.
  assert.throws(()=>cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:Array(65536).fill('H')},parameters}),{code:'resource_limit'});
  const count=Math.floor((1048576-1)/('upright:left:flat:raised'.length+3));
  assert.throws(()=>cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:Array(count).fill('H')},parameters}),{code:'resource_limit',operation:'symbol.alphabet.encode'});
});


test('bounded substitution and explainable families execute Core reports in Wasm',async()=>{
  const fixtures=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json',import.meta.url),'utf8'));
  const ids=new Set(['analysis.substitution.solve','analysis.family.suggest']);
  const cases=fixtures.filter(r=>ids.has(r.operation));
  assert.equal(cases.length,2);
  for(const row of cases) assert.deepEqual(cube.execute({operation:row.operation,version:1,inputs:row.inputs,parameters:row.parameters}),row.outputs);
  const request={profile:'observed_bijection_v1',alphabet:'ABCDEFGHIJKLMNOPQRSTUVWXYZ',ciphertext:'αBBα',outside:'preserve',aliases:[{alias:'α',symbol:'A'}],dictionary:{segmentation:'unicode_whitespace',words:['NOON','DEED','NOON']}};
  const solve=request=>cube.execute({operation:'analysis.substitution.solve',version:1,inputs:{request}}).report;
  const result=solve(request);
  assert.deepEqual(result.candidates.map(c=>c.plaintext),['NOON','DEED']);
  assert.equal(result.candidates[0].key_count,'620448401733239439360000');
  assert.deepEqual(result.dictionary_classes[0].input_indices,[0,2]);
  assert.equal(solve({...request,limits:{max_work:0}}).status.stop,'work_limit');
  assert.equal(solve({...request,limits:{max_results:1}}).status.stop,'result_limit');
  assert.throws(()=>solve({...request,limits:{max_work:1048577}}),{code:'invalid_input'});
  assert.throws(()=>solve({...request,alphabet:'ABC'}),{code:'invalid_input'});
  assert.throws(()=>solve({...request,alphabet:'A'.repeat(26)}),{code:'invalid_parameter'});
  const report=cube.execute({operation:'analysis.family.suggest',version:1,inputs:{request:{profile:'ascii_classical_evidence_v1',bytes:[0,255,65]}}}).report;
  assert.deepEqual(report.suggestions,[]);
  assert.ok(report.eligibility_failures.includes('outside_ascii_profile'));
});

test('five Hagelin profiles execute their manual answer and distinct advancement in actual Wasm', async () => {
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/hagelin_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,text,settings)=>cube.execute({operation:`cipher.hagelin.${direction}`,version:1,inputs:{text,settings}});
 assert.equal(execute('encode',v.text,v.settings).text,v.expected);
 assert.equal(execute('decode',v.expected,v.settings).text,v.text);
 const cases=[['c36_fixed25_bouch_v1',[25,23,21,19,17],[3,3,3,3,3]],['c38_standard27_v1',[26,25,23,21,19,17],[3,3,3,3,3,3]],['m209_standard_v1',[26,25,23,21,19,17],[3,3,3,3,3,3]],['c52_c2_27_v1',[26,25,46,42,38,34],[3,3,6,6,6,6]],['cx52_m27_late_v1',[47,47,47,47,47,47],[3,0,0,0,0,0]]];
 for(const [profile,lengths,state] of cases) {
  const settings={profile,pins:lengths.map(n=>'0'.repeat(n)),initial_state:lengths.map(()=>0)};
  if(!profile.startsWith('c36_'))settings.lugs=Array.from({length:27},()=>Array(6).fill(false));
  if(profile!=='m209_standard_v1')settings.slide=0;
  const expected=profile==='m209_standard_v1'?'ZYX':'AZY';
  const result=execute('encode','ABC',settings);
  assert.equal(result.text,expected);assert.deepEqual(result.final_state,state);
  assert.equal(execute('decode',expected,settings).text,'ABC');
  assert.deepEqual(execute('encode','',settings).final_state,settings.initial_state);
  const first=execute('encode','A',settings);
  const second=execute('encode','BC',{...settings,initial_state:first.final_state});
  assert.equal(first.text+second.text,expected);assert.deepEqual(second.final_state,state);
  assert.throws(()=>execute('encode','SECRET_KEY',settings),{code:'invalid_input'});
  assert.throws(()=>execute('encode','A'.repeat(65537),{...settings,trace:true}),{code:'resource_limit'});
  assert.equal(execute('encode','A'.repeat(262144),settings).text.length,262144);
  assert.throws(()=>execute('encode','A'.repeat(1048577),settings),{code:'resource_limit'});
 }
});

test('T52a/b uses the primary pluggable switch network and inverse-before-XOR through actual Wasm',async()=>{
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/t52ab_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,codes,settings)=>cube.execute({operation:`cipher.t52ab.${direction}`,version:1,inputs:{codes,settings}});
 const result=execute('encode',v.codes,v.settings);
 assert.deepEqual(result.codes,v.expected);assert.deepEqual(result.trace[0].permutation,[4,1,2,3,0]);
 assert.deepEqual(execute('decode',v.expected,v.settings).codes,v.codes);
 const codes=Array.from({length:32},(_,i)=>i),settings={...v.settings,trace:false};
 const whole=execute('encode',codes,settings);
 assert.deepEqual(execute('decode',whole.codes,settings).codes,codes);
 const first=execute('encode',codes.slice(0,11),settings);
 const second=execute('encode',codes.slice(11),{...settings,initial_state:first.final_state});
 assert.deepEqual([...first.codes,...second.codes],whole.codes);assert.deepEqual(second.final_state,whole.final_state);
 assert.deepEqual(execute('encode',[],settings).final_state,settings.initial_state);
 for(const codes of [[32],[-1],[true]])assert.throws(()=>execute('encode',codes,settings),{code:'invalid_input'});
 assert.throws(()=>execute('encode',[0],{...settings,kt:true}),{code:'invalid_input'});
 const request={operation:'cipher.t52ab.encode',version:1,inputs:{codes:[],settings},parameters:{}};
 const maximum=cube.limits.direct.documentNodes-documentNodeCount(request);
 assert.equal(execute('encode',Array(maximum).fill(31),settings).codes.length,maximum);
 assert.throws(()=>execute('encode',Array(maximum+1).fill(31),settings),{code:'resource_limit'});
 assert.throws(()=>execute('encode',Array(1048577).fill(0),settings),{code:'resource_limit'});
 assert.throws(()=>execute('encode',Array(65537).fill(0),{...settings,trace:true}),{code:'resource_limit'});
});

test('NEMA Model45 operational and training mechanisms retain physical windows in actual Wasm',async()=>{
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/nema_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,text,settings)=>cube.execute({operation:`cipher.nema.${direction}`,version:1,inputs:{text,settings}});
 const result=execute('encode',v.text,v.settings);
 assert.equal(result.text,v.expected);assert.deepEqual(result.final_state,v.expected_final_state);
 assert.equal(execute('decode',v.expected,v.settings).text,v.text);
 const steps=execute('encode','A'.repeat(26),{...v.settings,initial_state:{windows:'AAAAAAAAAA'}});
 assert.deepEqual(steps.trace.map(r=>r.state.windows),v.step_windows.slice(1));
 const training={...v.settings,profile:'nema_model45_training_v1',drive_wheels:[16,19,20,21],trace:false};
 const text='ABCDEFGHIJKLMNOPQRSTUVWXYZ';
 assert.equal(execute('decode',execute('encode',text,training).text,training).text,text);
 const first=execute('encode',text.slice(0,9),training);
 const second=execute('encode',text.slice(9),{...training,initial_state:first.final_state});
 assert.equal(first.text+second.text,execute('encode',text,training).text);
 assert.deepEqual(execute('encode','',training).final_state,training.initial_state);
 assert.throws(()=>execute('encode','A',{...training,rotors:['A','B','C','F']}),{code:'invalid_parameter'});
 assert.throws(()=>execute('encode','PRIVATE_TEXT',training),{code:'invalid_input'});
 assert.equal(execute('encode','A'.repeat(262144),training).text.length,262144);
 assert.throws(()=>execute('encode','A'.repeat(1048577),training),{code:'resource_limit'});
 assert.throws(()=>execute('encode','A'.repeat(65537),{...training,trace:true}),{code:'resource_limit'});
});


test('configured Typex Mk22 preserves independent reversed-core checks and stationary windows in actual Wasm',async()=>{
 const vectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/typex_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,text,settings)=>cube.execute({operation:`cipher.typex.${direction}`,version:1,inputs:{text,settings}});
 for(const v of vectors){
  const r=execute('encode',v.text,v.settings);assert.equal(r.text,v.expected);assert.deepEqual(r.final_state,v.expected_final_state);assert.equal(execute('decode',r.text,v.settings).text,v.text);
  const first=execute('encode',v.text.slice(0,9),v.settings);const second=execute('encode',v.text.slice(9),{...v.settings,initial_state:first.final_state});assert.equal(first.text+second.text,v.expected);assert.deepEqual(second.final_state,r.final_state);
  assert.deepEqual(execute('encode','',v.settings).final_state,v.settings.initial_state);
 }
 const settings={...vectors[0].settings,trace:false};
 assert.throws(()=>execute('encode','A',{...settings,reflector:'ABCDEFGHIJKLMNOPQRSTUVWXYZ'}),{code:'invalid_parameter'});
 assert.throws(()=>execute('encode','a',settings),{code:'invalid_input'});
 assert.equal(execute('encode','A'.repeat(262144),settings).text.length,262144);
 assert.throws(()=>execute('encode','A'.repeat(1048577),settings),{code:'resource_limit'});
 assert.throws(()=>execute('encode','A'.repeat(65537),{...settings,trace:true}),{code:'resource_limit'});
});


test('SIGABA preserves independent source results, physical state and explicit continuation in actual Wasm',async()=>{
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/sigaba_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,text,settings)=>cube.execute({operation:`cipher.sigaba.${direction}`,version:1,inputs:{text,settings}});
 const r=execute('encode',v.text,v.settings);assert.equal(r.text,v.expected);assert.deepEqual(r.final_state,v.expected_final_state);assert.equal(execute('decode',r.text,v.settings).text,v.text);
 const first=execute('encode',v.text.slice(0,9),v.settings);const second=execute('encode',v.text.slice(9),{...v.settings,initial_state:first.final_state});assert.equal(first.text+second.text,v.expected);assert.deepEqual(second.final_state,r.final_state);assert.deepEqual(execute('encode','',v.settings).final_state,v.settings.initial_state);
 const settings={...v.settings,trace:false};assert.equal(execute('encode','A'.repeat(262144),settings).text.length,262144);assert.throws(()=>execute('encode','A'.repeat(1048577),settings),{code:'resource_limit'});assert.throws(()=>execute('encode','A'.repeat(65537),{...settings,trace:true}),{code:'resource_limit'});assert.throws(()=>execute('encode','PRIVATE_TEXT',settings),{code:'invalid_input'});
});


test('KL7 preserves independent source results, physical state and explicit continuation in actual Wasm',async()=>{
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/kl7_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,text,settings)=>cube.execute({operation:`cipher.kl7.${direction}`,version:1,inputs:{text,settings}});
 const r=execute('encode',v.text,v.settings);assert.equal(r.text,v.expected);assert.deepEqual(r.final_state,v.expected_final_state);assert.equal(execute('decode',r.text,v.settings).text,v.text);
 const first=execute('encode',v.text.slice(0,9),v.settings);const second=execute('encode',v.text.slice(9),{...v.settings,initial_state:first.final_state});assert.equal(first.text+second.text,v.expected);assert.deepEqual(second.final_state,r.final_state);assert.deepEqual(execute('encode','',v.settings).final_state,v.settings.initial_state);
 const settings={...v.settings,trace:false};assert.equal(execute('encode','A'.repeat(262144),settings).text.length,262144);assert.throws(()=>execute('encode','A'.repeat(1048577),settings),{code:'resource_limit'});assert.throws(()=>execute('encode','A'.repeat(65537),{...settings,trace:true}),{code:'resource_limit'});assert.throws(()=>execute('encode','PRIVATE_TEXT',settings),{code:'invalid_input'});
});

test('FIALKA executes independent source answers and stateful boundaries in actual Wasm',async()=>{
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/fialka_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,input,settings)=>cube.execute({operation:`cipher.fialka.${direction}`,version:1,inputs:{codes:input,settings}});
 const r=execute('encode',v.codes,v.settings);assert.deepEqual(r.codes,v.expected);assert.deepEqual(r.final_state,v.expected_final_state);assert.deepEqual(execute('decode',r.codes,v.settings).codes,v.codes);
 const first=execute('encode',v.codes.slice(0,9),v.settings);const second=execute('encode',v.codes.slice(9),{...v.settings,initial_state:first.final_state});
 assert.deepEqual([...first.codes,...second.codes],v.expected);assert.deepEqual(second.final_state,r.final_state);
 assert.deepEqual(execute('encode',[],v.settings).final_state,v.settings.initial_state);
 const settings={...v.settings,trace:false};
 const request={operation:'cipher.fialka.encode',version:1,inputs:{codes:[],settings},parameters:{}};const maximum=cube.limits.direct.documentNodes-documentNodeCount(request);assert.equal(execute('encode',Array(maximum).fill(0),settings).codes.length,maximum);assert.throws(()=>execute('encode',Array(maximum+1).fill(0),settings),{code:'resource_limit'});
 assert.throws(()=>execute('encode',[30],settings),{code:'invalid_input'});
});

test('RED executes independent source answers and stateful boundaries in actual Wasm',async()=>{
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/red_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,input,settings)=>cube.execute({operation:`cipher.red.${direction}`,version:1,inputs:{text:input,settings}});
 const r=execute('encode',v.text,v.settings);assert.deepEqual(r.text,v.expected);assert.deepEqual(r.final_state,v.expected_final_state);assert.deepEqual(execute('decode',r.text,v.settings).text,v.text);
 const first=execute('encode',v.text.slice(0,9),v.settings);const second=execute('encode',v.text.slice(9),{...v.settings,initial_state:first.final_state});
 assert.deepEqual(first.text+second.text,v.expected);assert.deepEqual(second.final_state,r.final_state);
 assert.deepEqual(execute('encode','',v.settings).final_state,v.settings.initial_state);
 const settings={...v.settings,trace:false};
 assert.equal(execute('encode','A'.repeat(262144),settings).text.length,262144);assert.throws(()=>execute('encode','A'.repeat(1048577),settings),{code:'resource_limit'});assert.throws(()=>execute('encode','A'.repeat(65537),{...settings,trace:true}),{code:'resource_limit'});
 assert.throws(()=>execute('encode','PRIVATE_TEXT',settings),{code:'invalid_input'});
});

test('PURPLE executes independent source answers and stateful boundaries in actual Wasm',async()=>{
 const v=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/ciphers/mechanical/purple_vectors.json',import.meta.url),'utf8'));
 const execute=(direction,input,settings)=>cube.execute({operation:`cipher.purple.${direction}`,version:1,inputs:{text:input,settings}});
 const r=execute('encode',v.text,v.settings);assert.deepEqual(r.text,v.expected);assert.deepEqual(r.final_state,v.expected_final_state);assert.deepEqual(execute('decode',r.text,v.settings).text,v.text);
 const first=execute('encode',v.text.slice(0,9),v.settings);const second=execute('encode',v.text.slice(9),{...v.settings,initial_state:first.final_state});
 assert.deepEqual(first.text+second.text,v.expected);assert.deepEqual(second.final_state,r.final_state);
 assert.deepEqual(execute('encode','',v.settings).final_state,v.settings.initial_state);
 const settings={...v.settings,trace:false};
 assert.equal(execute('encode','A'.repeat(262144),settings).text.length,262144);assert.throws(()=>execute('encode','A'.repeat(1048577),settings),{code:'resource_limit'});assert.throws(()=>execute('encode','A'.repeat(65537),{...settings,trace:true}),{code:'resource_limit'});
 assert.throws(()=>execute('encode','PRIVATE_TEXT',settings),{code:'invalid_input'});
});
