import { spawnSync } from 'node:child_process';
import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { generateDirectTypes } from './generate-types.mjs';

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const workspace = resolve(packageRoot, '../..');
const development = process.argv.includes('--dev');
const profile = development ? 'debug' : 'release';
const target = process.env.CARGO_TARGET_DIR
  ? resolve(workspace, process.env.CARGO_TARGET_DIR)
  : join(workspace, 'target');

function run(command, args) {
  const result = spawnSync(command, args, { cwd: workspace, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

run(process.env.CARGO ?? 'cargo', [
  'build', '--locked', '--package', 'cube-javascript-wasm', '--target', 'wasm32-unknown-unknown',
  ...(development ? [] : ['--release']),
]);
const output = join(packageRoot, 'dist/wasm');
await mkdir(output, { recursive: true });
run(process.env.WASM_BINDGEN ?? 'wasm-bindgen', [
  join(target, 'wasm32-unknown-unknown', profile, 'cube_javascript_wasm.wasm'),
  '--target', 'web', '--out-name', 'cube_wasm', '--out-dir', output,
]);
await writeFile(join(output, 'package.json'), '{"type":"module"}\n');
await cp(join(packageRoot, 'src'), join(packageRoot, 'dist'), { recursive: true });
await cp(join(workspace, 'LICENSE'), join(packageRoot, 'dist/LICENSE'));
await cp(join(workspace, 'THIRD_PARTY_NOTICES'), join(packageRoot, 'dist/THIRD_PARTY_NOTICES'));
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const git = args => {
  const result = spawnSync('git', args, { cwd: workspace, encoding: 'utf8' });
  if (result.status !== 0) throw new Error('Build identity requires the source Git checkout.');
  return result.stdout.trim();
};
const sourceHash = createHash('sha256');
const sourcePaths = git(['ls-files', '--cached', '--others', '--exclude-standard', '-z'])
  .split('\0').filter(path => path && (/^(crates\/cube-core\/|bindings\/javascript\/|vendor\/)/.test(path) || ['Cargo.toml', 'Cargo.lock'].includes(path))).sort();
for (const path of sourcePaths) {
  sourceHash.update(path).update('\0').update(await readFile(join(workspace, path))).update('\0');
}
const build = {
  formatVersion: 1,
  sdkVersion: JSON.parse(await readFile(join(packageRoot, 'package.json'), 'utf8')).version,
  coreRevision: git(['rev-parse', 'HEAD']),
  sourceDirty: git(['status', '--porcelain']).length !== 0,
  sourceSha256: sourceHash.digest('hex'),
  catalogSha256: sha256((await readFile(join(workspace, 'contracts/v1/catalog.json'), 'utf8')).trim()),
  wasmSha256: sha256(await readFile(join(output, 'cube_wasm_bg.wasm'))),
};
await writeFile(join(packageRoot, 'dist/build-info.mjs'), `export const build = ${JSON.stringify(build)};\n`);
const catalog = JSON.parse(await readFile(join(workspace, 'contracts/v1/catalog.json'), 'utf8'));
if (catalog.engine_version !== build.sdkVersion) throw new Error('SDK package version must match the canonical Core engine version.');
const pipelineIds = catalog.operations.filter(operation => operation.pipeline_adapter !== null).map(operation => JSON.stringify(operation.operation));
await writeFile(join(packageRoot, 'dist/operation-ids.d.ts'), `// Generated from the canonical catalog; execution still requires Core preflight.\nexport type PipelineOperationId = ${pipelineIds.join(' | ')};\n`);
for (const entry of ['node', 'browser']) {
  await writeFile(join(packageRoot, `dist/${entry}.d.mts`), "export * from './index.js';\n");
}
const wasm = await import(pathToFileURL(join(output, 'cube_wasm.js')));
await wasm.default({ module_or_path: await readFile(join(output, 'cube_wasm_bg.wasm')) });
const snapshot = JSON.parse(wasm.catalog_snapshot_json());
if (snapshot.sha256 !== build.catalogSha256 || snapshot.json.trim() !== (await readFile(join(workspace,'contracts/v1/catalog.json'),'utf8')).trim()) {
  throw new Error('Actual Wasm catalog differs from canonical Core declarations.');
}
const executors=JSON.parse(wasm.executors_json());
const metadataJson=JSON.stringify({formatVersion:1,sourceSha256:build.sourceSha256,wasmSha256:build.wasmSha256,catalogSha256:build.catalogSha256,catalogJson:snapshot.json,contracts:snapshot.contracts,executors});
build.metadataSha256=sha256(metadataJson);
await writeFile(join(packageRoot,'dist/runtime-metadata.mjs'),`// Generated from this exact Wasm artifact; never edit.\nexport const metadata = ${metadataJson};\n`);
await writeFile(join(packageRoot,'dist/build-info.mjs'),`export const build = ${JSON.stringify(build)};\n`);
await generateDirectTypes(join(packageRoot, 'dist/direct-operations.d.ts'), executors, catalog);
