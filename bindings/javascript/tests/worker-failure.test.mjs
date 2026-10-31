import assert from 'node:assert/strict';
import test from 'node:test';
import { createWorkerCube } from '../dist/worker-client.mjs';
import { build } from '../dist/build-info.mjs';

test('a reported runtime trap discards the worker realm before the next invocation', async () => {
  // Protocol test only: actual browser lifecycle is qualified separately.
  const original = globalThis.Worker;
  const instances = [];
  globalThis.Worker = class {
    constructor() { instances.push(this); }
    terminate() { this.terminated = true; }
    postMessage(message) {
      queueMicrotask(() => this.onmessage({ data: message.method === 'initialize'
        ? { id: message.id, value: { build, limits: {}, pipelineAdapters: [], executors: [] } }
        : instances.length === 1 ? { id: message.id, error: { code: 'runtime_failure', message: 'The runtime failed.' } }
          : { id: message.id, value: { output: 'ready' } } }));
    }
  };
  try {
    const worker = await createWorkerCube();
    await assert.rejects(worker.execute({}), { code: 'runtime_failure' });
    const next = await worker.execute({});
    assert.equal(instances[0].terminated, true);
    assert.equal(instances.length, 2);
    assert.equal(next.value.output, 'ready');
    worker.terminate();
  } finally { globalThis.Worker = original; }
});
