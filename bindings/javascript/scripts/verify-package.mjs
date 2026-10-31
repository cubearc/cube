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
  await writeFile(join(directory, 'consumer.mjs'), `
import assert from 'node:assert/strict';
import { createCube, CubeError } from '@cubearc/cube';
const cube = await createCube();
assert.equal(await createCube(), cube);
const output = cube.execute({operation:'encoding.base64.decode',version:1,inputs:{input:'AP8='}}).output;
assert.deepEqual(output, Uint8Array.of(0,255));
const definition = {format_version:1,engine_api:1,input:{source:'slot',name:'message',type:'text'},output:{type:'text'},steps:[{operation:'encoding.base64.decode',version:1},{operation:'text.utf8.decode',version:1}]};
const run = cube.runRecipe(cube.recipeJson(definition),{message:{type:'text',data:'Q2Fmw6k='}},{captures:[0]});
assert.equal(run.output.data,'Café');
assert.deepEqual(run.intermediates['0'].data,Uint8Array.of(67,97,102,195,169));
assert.equal(JSON.parse(cube.catalogJson()).operations.length,329);
assert.ok(CubeError.prototype instanceof Error);
await import('@cubearc/cube/browser');
await import('@cubearc/cube/worker');
console.log('Installed package: Node '+process.version+'; exact bytes, recipe, catalog and public exports passed.');
`);
  run(process.execPath, ['consumer.mjs']);
  await writeFile(join(directory, 'consumer.mts'), `import { createCube } from '@cubearc/cube';\nconst cube = await createCube();\nconst value: Uint8Array = cube.execute({operation:'encoding.base64.decode',version:1,inputs:{input:'AP8='}}).output;\nvoid value;\n`);
  const compiler = resolve(dirname(fileURLToPath(import.meta.url)), '../node_modules/typescript/bin/tsc');
  await readFile(compiler);
  run(process.execPath, [compiler, '--strict', '--noEmit', '--target', 'ES2022', '--module', 'NodeNext', '--moduleResolution', 'NodeNext', 'consumer.mts']);
  const assets = join(directory, 'node_modules/@cubearc/cube');
  for (const file of ['dist/wasm/cube_wasm_bg.wasm', 'dist/LICENSE', 'dist/THIRD_PARTY_NOTICES', 'examples/static/index.html', 'examples/node.mjs', 'examples/node.mts']) await readFile(join(assets, file));
  const notices = await readFile(join(assets, 'dist/THIRD_PARTY_NOTICES'));
  if (!notices.equals(await readFile(new URL('../../../THIRD_PARTY_NOTICES', import.meta.url)))) throw new Error('Packaged third-party notices differ from the reviewed source.');
  run(process.execPath, [join(assets, 'examples/node.mjs')]);
  await writeFile(join(directory, 'example.mts'), await readFile(join(assets, 'examples/node.mts')));
  run(process.execPath, [compiler, '--strict', '--target', 'ES2022', '--module', 'NodeNext', '--moduleResolution', 'NodeNext', '--outDir', 'compiled', 'example.mts']);
  run(process.execPath, [join(directory, 'compiled/example.mjs')]);
} finally { await rm(directory, { recursive: true, force: true }); }
