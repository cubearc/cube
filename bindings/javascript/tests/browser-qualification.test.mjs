import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { readFile, rm, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { qualificationReport } from '../scripts/qualification-library.mjs';
import { build } from '../dist/build-info.mjs';

const require = createRequire(import.meta.url);
const browserType = require(process.env.PLAYWRIGHT_MODULE ?? 'playwright')[process.env.BROWSER_ENGINE ?? 'chromium'];
const files = new Map([
  ['/docs/playground/wasm/cube_wasm.js', ['text/javascript', new URL('../dist/wasm/cube_wasm.js', import.meta.url)]],
  ['/docs/playground/wasm/cube_wasm_bg.wasm', ['application/wasm', new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url)]],
  ['/docs/playground/catalog.json', ['application/json', new URL('../../../contracts/v1/catalog.json', import.meta.url)]],
  ['/docs/playground/fixtures.json', ['application/json', new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url)]],
]);
for (const name of ['browser.mjs','canonical-input.mjs','errors.mjs','transport.mjs','loading.mjs','build-info.mjs','runtime-metadata.mjs','worker.mjs','worker-client.mjs']) {
  files.set(`/docs/playground/${name}`,['text/javascript',new URL(`../dist/${name}`,import.meta.url)]);
}
const html = `<!doctype html><meta charset="utf-8"><title>Cube Wasm qualification</title>
<script type="module">
import init, * as wasm from './wasm/cube_wasm.js';
import {metadata} from './runtime-metadata.mjs';
globalThis.qualification = (async () => {
  await init();
  const {createWorkerCube}=await import('./worker-client.mjs');
  globalThis.qualificationWorker=await createWorkerCube();
  const original = Uint8Array.of(0, 255, 128, 1);
  const encoded = wasm.encode_base64(original);
  const bytes = Array.from(wasm.decode_base64(encoded));
  let failure;
  try { wasm.decode_base64('not-valid!'); }
  catch (error) { failure = JSON.parse(error); }
  const canonicalJson = await fetch('./catalog.json').then(response => response.text());
  const actualJson = (await globalThis.qualificationWorker.catalogJson()).value;
  const snapshot = {json:actualJson,sha256:globalThis.qualificationWorker.build.catalogSha256,contracts:metadata.contracts};
  const actual = JSON.parse(actualJson);
  const recipe = {
    format_version: 1, engine_api: 1,
    input: {source: 'inline', value: {type: 'text', data: 'Q2Fmw6k='}},
    output: {type: 'text'},
    steps: [
      {operation: 'encoding.base64.decode', version: 1},
      {operation: 'text.utf8.decode', version: 1},
    ],
  };
  const result = JSON.parse(wasm.run_recipe_json(JSON.stringify(recipe), '{}', Uint32Array.of(0)));
  const adapters = globalThis.qualificationWorker.pipelineAdapters;
  const executors = globalThis.qualificationWorker.executors;
  return {encoded, bytes, original: Array.from(original), failure, result,
    exactCatalog: actualJson.trim() === canonicalJson.trim(),
    adapters,
    executors,
    snapshot,
    limits: globalThis.qualificationWorker.limits,
    operationIds: actual.operations.map(operation => operation.operation)};
})();
</script>`;

// This is the aggregate catalog audit, not an individual operation deadline.
// Firefox CI completed 300 known answers before the former four-minute cap.
// Keep SDK initialization and each worker invocation at their existing 30 seconds.
test('actual browser initializes a packaged SDK worker under a non-root path and runs every Core binding', {timeout:600000}, async (t) => {
  const destination = process.env.CUBE_QUALIFICATION_REPORT;
  if (destination) await rm(destination, { force: true });
  const wasmBytes = await readFile(files.get('/docs/playground/wasm/cube_wasm_bg.wasm')[1]);
  assert.equal(createHash('sha256').update(wasmBytes).digest('hex'), build.wasmSha256);
  const requests = [];
  const server = createServer(async (request, response) => {
    const path = new URL(request.url, 'http://localhost').pathname;
    requests.push(path);
    if (path === '/docs/playground/') {
      response.writeHead(200, { 'Content-Type': 'text/html' }).end(html);
      return;
    }
    const file = files.get(path);
    if (!file) { response.writeHead(404).end(); return; }
    try {
      response.writeHead(200, { 'Content-Type': file[0] }).end(await readFile(file[1]));
    } catch { response.writeHead(500).end(); }
  });
  server.listen(0, '127.0.0.1');
  await new Promise((resolve, reject) => {
    server.once('listening', resolve);
    server.once('error', reject);
  });
  t.after(() => new Promise((resolve) => server.close(resolve)));
  const browser = await browserType.launch({
    headless: true,
    ...(process.env.BROWSER_EXECUTABLE ? { executablePath: process.env.BROWSER_EXECUTABLE } : {}),
  });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const external = [];
  const origin = `http://127.0.0.1:${server.address().port}`;
  page.on('request', (request) => { if (!request.url().startsWith(origin)) external.push(request.url()); });
  await page.goto(`${origin}/docs/playground/`);
  const actual = await page.evaluate(() => globalThis.qualification);
  assert.equal(actual.encoded, 'AP+AAQ==');
  assert.deepEqual(actual.bytes, [0, 255, 128, 1]);
  assert.deepEqual(actual.original, [0, 255, 128, 1]);
  assert.equal(actual.failure.code, 'invalid_input');
  assert.equal(actual.failure.operation, 'encoding.base64.decode');
  assert.ok(!actual.failure.message.includes('not-valid'));
  assert.deepEqual(actual.result.output, { type: 'text', data: 'Café' });
  assert.deepEqual(actual.result.intermediates['0'], { type: 'bytes', base64: 'Q2Fmw6k=' });
  const fixtures = JSON.parse(await readFile(files.get('/docs/playground/fixtures.json')[1], 'utf8'));
  const alphabetVectors=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/symbols/alphabet/vectors.json',import.meta.url),'utf8'));
  for(const row of alphabetVectors) {
    const results=await page.evaluate(async row=> {
      const parameters={profile:row.profile};
      const encoded=await globalThis.qualificationWorker.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:row.units},parameters});
      const decoded=await globalThis.qualificationWorker.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:row.tokens},parameters});
      return [encoded.value.tokens,decoded.value.readings];
    },row);
    assert.deepEqual(results,[row.tokens,row.units.map((unit,token_index)=>({token_index,alternatives:row.alternatives?.[token_index]??[unit]}))]);
  }
  const alphabetAliases=JSON.parse(await readFile(new URL('../../../crates/cube-core/src/symbols/alphabet/aliases.json',import.meta.url),'utf8'));
  for(const row of alphabetAliases) {
    const result=await page.evaluate(async row=> {
      const {value}=await globalThis.qualificationWorker.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[row.token]},parameters:{profile:row.profile}});
      return {tokens:value.tokens,readings:value.readings};
    },row);
    assert.deepEqual(result,{tokens:[row.token],readings:[{token_index:0,alternatives:row.alternatives}]});
  }
  let completedDirect = 0;
  const report = await qualificationReport({
    snapshot: actual.snapshot,
    canonicalJson: await readFile(files.get('/docs/playground/catalog.json')[1], 'utf8'),
    fixtures, build, executors: actual.executors, adapters: actual.adapters, limits: actual.limits,
    runtime: { kind: 'browser-wasm', transport: 'public-sdk-worker', identityEvidence: 'Live Core contract identities generated from these exact Wasm bytes during packaging; browser verifies both metadata and Wasm digests and executes every advertised path', engine: process.env.BROWSER_ENGINE ?? 'chromium', version: browser.version(), platform: process.platform, architecture: process.arch },
    driver: {
      executeDirect: async (request,binding) => {
        const output = await page.evaluate(async ({request,binding}) => {
        const wasm=await import('./wasm/cube_wasm.js');
        const inputs=Object.fromEntries(Object.entries(request.inputs).map(([name,value])=>[name,
          binding.inputs[name]==='bytes'?wasm.decode_base64(value.base64):
          binding.inputs[name]==='byte_array'?Uint8Array.from(value):value]));
        const {value:actual}=await globalThis.qualificationWorker.execute({...request,inputs});
        return Object.fromEntries(Object.entries(actual).map(([name,value])=>[name,
          binding.outputs[name]==='bytes'?{base64:wasm.encode_base64(value)}:
          binding.outputs[name]==='byte_array'?Array.from(value):value]));
        }, {request,binding});
        if (++completedDirect % 50 === 0) console.log(`Executed ${completedDirect} direct known answers in the actual browser SDK worker.`);
        return output;
      },
      runRecipe: definition => page.evaluate(async definition => {
        const wasm=await import('./wasm/cube_wasm.js');
        const {value}=await globalThis.qualificationWorker.runRecipe(definition,{}, {captures:[0]});
        const wire=v=>v.type==='bytes'?{type:'bytes',base64:wasm.encode_base64(v.data)}:v;
        return {output:wire(value.output),intermediates:{'0':wire(value.intermediates['0'])}};
      }, definition),
    },
  });
  assert.equal(actual.exactCatalog, true);
  assert.ok(requests.includes('/docs/playground/wasm/cube_wasm_bg.wasm'));
  assert.deepEqual(external, []);
  if (destination) await writeFile(destination, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`Browser: ${browser.version()}; ${actual.operationIds.length} canonical contracts verified; ${report.coverage.pipeline} pipeline adapters and ${report.coverage.direct} direct executors executed`);
});
