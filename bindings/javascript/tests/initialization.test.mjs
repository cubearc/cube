import assert from 'node:assert/strict';
import { readFile, cp, mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import { createCube } from '../dist/browser.mjs';

test('a mismatched valid Wasm module is rejected before instantiation and initialization can retry', async () => {
  const emptyModule = Uint8Array.of(0, 97, 115, 109, 1, 0, 0, 0);
  await assert.rejects(createCube({ wasm: emptyModule }), { code: 'incompatible_runtime' });
  const bytes = await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url));
  const first = createCube({ wasm: bytes });
  const second = createCube({ wasm: bytes });
  bytes.fill(0); // Initialization must already own a frozen copy.
  assert.equal(first, second);
  const cube = await first;
  assert.equal(cube.execute({ operation: 'encoding.base64.encode', version: 1, inputs: { input: Uint8Array.of(255) } }).output, '/w==');
  assert.equal((await second), cube);
  await assert.rejects(createCube({ wasm: emptyModule }), { code: 'initialization_conflict' });
  assert.match(cube.build.wasmSha256, /^[0-9a-f]{64}$/);
  assert.equal(cube.build.sdkVersion, '0.18.0');
});

test('discovery metadata from a different artifact is rejected with the real Wasm unchanged', async () => {
  const temporary=await mkdtemp(join(tmpdir(),'cube-metadata-'));
  try {
    await cp(new URL('../dist/',import.meta.url),temporary,{recursive:true});
    const path=join(temporary,'runtime-metadata.mjs');
    const source=await readFile(path,'utf8');
    await writeFile(path,source.replace(/"wasmSha256":"[a-f0-9]{64}"/,'"wasmSha256":"'+'0'.repeat(64)+'"'));
    const {createCube:isolated}=await import(pathToFileURL(join(temporary,'node.mjs')));
    await assert.rejects(isolated(),{code:'incompatible_runtime'});
  } finally {await rm(temporary,{recursive:true,force:true});}
});

test('changed discovery data is rejected even when its declared build identities are unchanged', async () => {
  const temporary=await mkdtemp(join(tmpdir(),'cube-capability-data-'));
  try {
    await cp(new URL('../dist/',import.meta.url),temporary,{recursive:true});
    const path=join(temporary,'runtime-metadata.mjs');
    const source=await readFile(path,'utf8');
    const changed=source.replace('"operation":"cipher.book.decode"','"operation":"not.a.cube.operation"');
    assert.notEqual(changed,source);
    await writeFile(path,changed);
    const {createCube:isolated}=await import(pathToFileURL(join(temporary,'node.mjs')));
    await assert.rejects(isolated(),{code:'incompatible_runtime'});
  } finally {await rm(temporary,{recursive:true,force:true});}
});
