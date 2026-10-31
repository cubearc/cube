import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const require = createRequire(import.meta.url);
const browserType = require(process.env.PLAYWRIGHT_MODULE ?? 'playwright')[process.env.BROWSER_ENGINE ?? 'chromium'];
const modules = ['browser.mjs', 'errors.mjs', 'transport.mjs', 'canonical-input.mjs', 'loading.mjs', 'build-info.mjs','runtime-metadata.mjs', 'worker.mjs', 'worker-client.mjs', 'wasm/cube_wasm.js', 'wasm/cube_wasm_bg.wasm'];

test('browser SDK verifies static assets and dedicated workers terminate and restart honestly', { timeout: 60000 }, async t => {
  let exampleMissing = true;
  const server = createServer(async (request, response) => {
    response.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
    response.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
    const path = new URL(request.url, 'http://localhost').pathname;
    if (path === '/docs/playground/') {
      response.writeHead(200, { 'Content-Type': 'text/html' }).end('<!doctype html><meta charset="utf-8"><title>SDK integration</title>');
      return;
    }
    if (path === '/docs/sdk-example/examples/static/') {
      response.writeHead(200, { 'Content-Type': 'text/html' }).end(await readFile(new URL('../examples/static/index.html', import.meta.url)));
      return;
    }
    if (path === '/docs/sdk-example/dist/wasm/cube_wasm_bg.wasm' && exampleMissing) { exampleMissing = false; response.writeHead(404).end(); return; }
    const module = path.replace('/docs/playground/sdk/', '').replace('/docs/sdk-example/dist/', '');
    if (!modules.includes(module)) { response.writeHead(404).end(); return; }
    try {
      const bytes = await readFile(new URL(`../dist/${module}`, import.meta.url));
      // Intentionally use a non-streaming Wasm MIME type to test byte loading.
      response.writeHead(200, { 'Content-Type': module.endsWith('.wasm') ? 'application/octet-stream' : 'text/javascript' })
        .end(bytes);
    } catch { response.writeHead(404).end(); }
  });
  server.listen(0, '127.0.0.1');
  await new Promise((resolve, reject) => { server.once('listening', resolve); server.once('error', reject); });
  t.after(() => { server.closeAllConnections(); server.close(); });
  const browser = await browserType.launch({ headless: true, ...(process.env.BROWSER_EXECUTABLE ? { executablePath: process.env.BROWSER_EXECUTABLE } : {}) });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const origin = `http://127.0.0.1:${server.address().port}`;
  const external = [];
  page.on('request', request => { if (!request.url().startsWith(origin)) external.push(request.url()); });
  await page.goto(`${origin}/docs/playground/`);
  const result = await page.evaluate(async () => {
    const { createCube } = await import('./sdk/browser.mjs');
    let missing;
    try { await createCube({ wasm: new URL('./missing.wasm', location.href) }); }
    catch (error) { missing = error.code; }
    const cube = await createCube();
    const direct = cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: 'AP8=' } });
    const { createWorkerCube } = await import('./sdk/worker-client.mjs');
    const worker = await createWorkerCube();
    const original = typeof SharedArrayBuffer === 'function' ? new Uint8Array(new SharedArrayBuffer(4)) : new Uint8Array(4);
    original.set([0, 255, 128, 1]);
    const invocation = worker.execute({ operation: 'encoding.base64.encode', version: 1, inputs: { input: original } }, { context: { revision: 7 } });
    original.fill(1);
    const encoded = await invocation;
    const controller = new AbortController();
    const slow = worker.catalogJson({ signal: controller.signal }).then(() => 'completed', error => error.code);
    const busy = await worker.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: '' } }).then(() => 'unexpected-success', error => error.code);
    controller.abort();
    const stopped = await slow;
    const decoded = await worker.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: encoded.value.output } });
    worker.terminate();
    return { missing, direct: Array.from(direct.output), encoded, stopped, busy,
      decoded: Array.from(decoded.value.output), firstGeneration: encoded.generation, nextGeneration: decoded.generation,
      original: Array.from(original), adapterCount: worker.pipelineAdapters.length, sourceDirty: worker.build.sourceDirty };
  });
  assert.equal(result.missing, 'initialization_failed');
  assert.deepEqual(result.direct, [0, 255]);
  assert.equal(result.encoded.value.output, 'AP+AAQ==');
  assert.deepEqual(result.encoded.context, { revision: 7 });
  assert.equal(result.stopped, 'worker_terminated');
  assert.equal(result.busy, 'resource_limit');
  assert.ok(result.nextGeneration > result.firstGeneration);
  assert.deepEqual(result.decoded, [0, 255, 128, 1]);
  assert.deepEqual(result.original, [1, 1, 1, 1]);
  assert.equal(result.adapterCount, 53);
  await page.goto(`${origin}/docs/sdk-example/examples/static/`);
  await page.waitForFunction(() => document.querySelector('#status').textContent.includes('Run again to retry.'));
  await page.locator('#run').click();
  await page.getByText('2 exact bytes ready.', { exact: true }).waitFor();
  const [download] = await Promise.all([page.waitForEvent('download'), page.locator('#download').click()]);
  assert.deepEqual(new Uint8Array(await readFile(await download.path())), Uint8Array.of(0, 255));
  await page.locator('#stop').click();
  await page.getByText('Worker ended. Another run will initialize a fresh worker.', { exact: true }).waitFor();
  await page.locator('#run').click();
  await page.getByText('2 exact bytes ready.', { exact: true }).waitFor();
  assert.deepEqual(external, []);
  t.diagnostic(`Actual browser ${browser.version()}; Wasm integrity, non-root loading, worker ownership, termination and restart verified`);
});
