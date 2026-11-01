// Install an actual tarball without lifecycle scripts or a source checkout.
import { mkdtemp, writeFile, rm, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const tarball = resolve(process.argv[2] ?? '');
if (!process.argv[2]) throw new Error('Provide the packed SDK tarball.');
const directory = await mkdtemp(join(tmpdir(), 'cube-sdk-consumer-'));
function run(command, args) {
  const result = spawnSync(command, args, { cwd: directory, stdio: 'inherit', env: { ...process.env, npm_config_cache: join(directory, '.npm-cache') }, shell: process.platform === 'win32' && command === 'npm' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error('Installed SDK consumer validation failed.');
}
try {
  await writeFile(join(directory, 'package.json'), '{"private":true,"type":"module"}');
  run('npm', ['install', '--offline', '--ignore-scripts', '--no-audit', '--no-fund', tarball]);
  await writeFile(join(directory, 'fixtures.json'), await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url)));
  await writeFile(join(directory, 'alphabet-vectors.json'), await readFile(new URL('../../../crates/cube-core/src/symbols/alphabet/vectors.json', import.meta.url)));
  await writeFile(join(directory, 'alphabet-aliases.json'), await readFile(new URL('../../../crates/cube-core/src/symbols/alphabet/aliases.json', import.meta.url)));
  await writeFile(join(directory, 'consumer.mjs'), `
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { isDeepStrictEqual } from 'node:util';
import { createCube, CubeError } from '@cubearc/cube';
const cube = await createCube();
assert.equal(await createCube(), cube);
const output = cube.execute({operation:'encoding.base64.decode',version:1,inputs:{input:'AP8='}}).output;
assert.deepEqual(output, Uint8Array.of(0,255));
const definition = {format_version:1,engine_api:1,input:{source:'slot',name:'message',type:'text'},output:{type:'text'},steps:[{operation:'encoding.base64.decode',version:1},{operation:'text.utf8.decode',version:1}]};
const run = cube.runRecipe(cube.recipeJson(definition),{message:{type:'text',data:'Q2Fmw6k='}},{captures:[0]});
assert.equal(run.output.data,'Café');
assert.deepEqual(run.intermediates['0'].data,Uint8Array.of(67,97,102,195,169));
const fixtures=JSON.parse(await readFile('./fixtures.json','utf8'));
assert.equal(JSON.parse(cube.catalogJson()).operations.length,fixtures.length);
assert.equal(cube.executors.length,fixtures.length);
for (const fixture of fixtures) {
  const binding=cube.executors.find(row=>row.operation===fixture.operation);
  assert.ok(binding,fixture.operation);
  const inputs=Object.fromEntries(Object.entries(fixture.inputs).map(([name,value])=>[name,
    binding.inputs[name]==='bytes'?Uint8Array.from(Buffer.from(value.base64,'base64')):
    binding.inputs[name]==='byte_array'?Uint8Array.from(value):value]));
  const outputs=cube.execute({operation:fixture.operation,version:fixture.operation_version,inputs,parameters:fixture.parameters});
  const wire=Object.fromEntries(Object.entries(outputs).map(([name,value])=>[name,
    binding.outputs[name]==='bytes'?{base64:Buffer.from(value).toString('base64')}:
    binding.outputs[name]==='byte_array'?Array.from(value):value]));
  assert.ok(isDeepStrictEqual(wire,fixture.outputs),fixture.operation+' packaged known answer');
}
const alphabetVectors=JSON.parse(await readFile('./alphabet-vectors.json','utf8'));
for(const row of alphabetVectors) {
  const parameters={profile:row.profile};
  assert.deepEqual(cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:row.units},parameters}).tokens,row.tokens);
  assert.deepEqual(cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:row.tokens},parameters}).readings,row.units.map((unit,token_index)=>({token_index,alternatives:row.alternatives?.[token_index]??[unit]})));
}
const alphabetAliases=JSON.parse(await readFile('./alphabet-aliases.json','utf8'));
for(const row of alphabetAliases) {
  const r=cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[row.token]},parameters:{profile:row.profile}});
  assert.deepEqual(r.tokens,[row.token]);
  assert.deepEqual(r.readings,[{token_index:0,alternatives:row.alternatives}]);
}
assert.ok(CubeError.prototype instanceof Error);
await import('@cubearc/cube/browser');
await import('@cubearc/cube/worker');
console.log('Installed package: Node '+process.version+'; all '+fixtures.length+' direct known answers, bytes, recipe and exports passed.');
`);
  run(process.execPath, ['consumer.mjs']);
  await writeFile(join(directory, 'consumer.mts'), `import { createCube } from '@cubearc/cube';\nconst cube = await createCube();\nconst value: Uint8Array = cube.execute({operation:'encoding.base64.decode',version:1,inputs:{input:'AP8='}}).output;\nconst book: string = cube.execute({operation:'cipher.book.decode',version:1,inputs:{input:'1:1,2:1'},parameters:{source:'another book',scheme:'word_character',index_base:1}}).output;\nif(book !== 'ab') throw new Error('Compiled TypeScript Book cipher mismatch');\nconst phrase: string=cube.execute({operation:'cipher.key_phrase.encode',version:1,inputs:{text:'BHQ'},parameters:{phrase:'GIVEMELIBERTYORGIVEMEDEATH'}}).text;\nif(phrase!=='III') throw new Error('Packaged phrase encoding mismatch');\nconst reading: string|null=cube.execute({operation:'cipher.key_phrase.decode',version:1,inputs:{text:'I'},parameters:{phrase:'GIVEMELIBERTYORGIVEMEDEATH'}}).text;\nif(reading!==null) throw new Error('Packaged inverse guessed an ambiguous letter');\nconst words: readonly string[]=cube.execute({operation:'cipher.arnold.encode',version:1,inputs:{pages:['A'],words:['A']},parameters:{profile:'arnold_paged_words_v1',source_id:'synthetic',index_base:1,choice:'first'}}).words;\nif(words[0]!=='A') throw new Error('Packaged whole-word code mismatch');\nconst numbers: readonly string[]=cube.execute({operation:'cipher.numbered_key.encode',version:1,inputs:{text:'AA'},parameters:{alphabet:'AB',key:'ABA',preparation:'exact',rotation:0,origin:'0',width:1,choice:'round_robin'}}).tokens;\nif(numbers.join(',')!=='0,2') throw new Error('Packaged repeated-key positions mismatch');\nconst headlines: readonly string[]=cube.execute({operation:'cipher.headlines.encode',version:1,inputs:{headlines:['','','','','']},parameters:{profile:'aca_base',hat:'APOTHECARY',key:'CHEMIST',setting:'DRUGS',punctuation:'preserve'}}).headlines;\nif(headlines.length!==5) throw new Error('Packaged five-headline profile mismatch');\nconst solresol: readonly string[]=cube.execute({operation:'symbol.solresol.encode',version:1,inputs:{terms:['non','oui','à']},parameters:{profile:'gajewski_1902_monosyllables_v1'}}).tokens;\nif(solresol.join(',')!=='do,si,fa') throw new Error('Packaged finite lexicon mismatch');\nconst glosses: readonly string[]=cube.execute({operation:'symbol.solresol.decode',version:1,inputs:{tokens:['do']},parameters:{profile:'gajewski_1902_monosyllables_v1'}}).readings[0].alternatives;\nif(glosses.length!==5) throw new Error('Packaged lexicon lost synonyms');\nconst primeTokens:readonly string[]=cube.execute({operation:'cipher.prime_number.encode',version:1,inputs:{text:'AB'},parameters:{profile:'indexed_primes_v1',index:{alphabet:'AB',first_index:1,max_prime:3}}}).tokens;\nif(primeTokens.join(',')!=='2,3') throw new Error('Packaged indexed primes mismatch');\nconst multiset=cube.execute({operation:'cipher.prime_multiplication.decode',version:1,inputs:{products:['6']},parameters:{profile:'prime_multiset_v1',index:{alphabet:'AB',first_index:1,max_prime:3},max_factor_checks:10}}).groups[0];\nconst exactProduct:string=multiset.product;\nconst multiplicity:number=multiset.counts[0].count;\nif(exactProduct!=='6'||multiplicity!==1||multiset.canonical_text!=='AB') throw new Error('Packaged prime multiset mismatch');\nconst residues:readonly string[]=cube.execute({operation:'cipher.modulo.encode',version:1,inputs:{text:'A'},parameters:{profile:'affine_residue_v1',alphabet:'AB',modulus:2,multiplier:'1',offset:'0',token_origin:1}}).tokens;\nif(residues[0]!=='1') throw new Error('Packaged modular origin mismatch');\nconst trits:readonly string[]=cube.execute({operation:'cipher.triliteral.encode',version:1,inputs:{text:'AB'},parameters:{profile:'ordered_ternary_v1',alphabet:'AB',trits:'012'}}).tokens;\nif(trits.join(',')!=='000,001') throw new Error('Packaged trit width mismatch');\nconst symbolic:readonly string[]=cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:['CH']},parameters:{profile:'aurebesh_ucsur_2016_v1'}}).tokens;\nif(symbolic[0].codePointAt(0)!==0xe8aa) throw new Error('Packaged symbolic digraph mismatch');\nconst info=cube.execute({operation:'file.info.inspect',version:2,inputs:{input:value}}).output;\nif(info.size !== 2 || info.regular_file !== null) throw new Error('Compiled TypeScript artifact inspection mismatch');\n`);
  const compiler = resolve(dirname(fileURLToPath(import.meta.url)), '../node_modules/typescript/bin/tsc');
  await readFile(compiler);
  run(process.execPath, [compiler, '--strict', '--target', 'ES2022', '--module', 'NodeNext', '--moduleResolution', 'NodeNext', '--outDir', 'compiled-consumer', 'consumer.mts']);
  run(process.execPath, [join(directory, 'compiled-consumer/consumer.mjs')]);
  const assets = join(directory, 'node_modules/@cubearc/cube');
  for (const file of ['dist/wasm/cube_wasm_bg.wasm', 'dist/LICENSE', 'dist/THIRD_PARTY_NOTICES', 'examples/static/index.html', 'examples/node.mjs', 'examples/node.mts']) await readFile(join(assets, file));
  const notices = await readFile(join(assets, 'dist/THIRD_PARTY_NOTICES'));
  if (!notices.equals(await readFile(new URL('../../../THIRD_PARTY_NOTICES', import.meta.url)))) throw new Error('Packaged third-party notices differ from the reviewed source.');
  run(process.execPath, [join(assets, 'examples/node.mjs')]);
  await writeFile(join(directory, 'example.mts'), await readFile(join(assets, 'examples/node.mts')));
  run(process.execPath, [compiler, '--strict', '--target', 'ES2022', '--module', 'NodeNext', '--moduleResolution', 'NodeNext', '--outDir', 'compiled', 'example.mts']);
  run(process.execPath, [join(directory, 'compiled/example.mjs')]);
} finally { await rm(directory, { recursive: true, force: true }); }
