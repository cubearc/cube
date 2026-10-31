import { CubeError } from './errors.mjs';
import { build } from './build-info.mjs';
import { immutable, integerDecimal } from './transport.mjs';
export { CubeError } from './errors.mjs';

function timeout(value) {
  if (!Number.isInteger(value) || value < 1 || value > 120000) throw new CubeError('invalid_parameter', 'Worker deadline must be 1..120000 milliseconds.');
  return value;
}

function snapshotBudget(input) {
  let bytes = 0;
  let nodes = 0;
  const visit = (value, depth) => {
    if (++nodes > 8192 || depth > 20) throw new CubeError('resource_limit', 'Worker request exceeds nesting or node bounds.');
    if (typeof value === 'string') bytes += value.length * 3;
    else if (typeof value === 'bigint') bytes += integerDecimal(value).length;
    else if (value instanceof Uint8Array) bytes += value.byteLength;
    else if (value && typeof value === 'object') {
      if (!Array.isArray(value) && Object.getPrototypeOf(value) !== Object.prototype) throw new CubeError('invalid_input', 'Use plain JSON values and explicit Uint8Array data.');
      for (const [key, item] of Object.entries(value)) { bytes += key.length * 3; visit(item, depth + 1); }
    }
    if (bytes > 2 * 1024 * 1024) throw new CubeError('resource_limit', 'Worker request exceeds its two-MiB conservative copy budget.');
  };
  visit(input, 0);
}

function ownedCopy(value) {
  if (value instanceof Uint8Array) return new Uint8Array(value);
  if (Array.isArray(value)) return value.map(ownedCopy);
  if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, ownedCopy(item)]));
  return value;
}

/** Dedicated worker, at most one pending invocation, no buffer transfers. */
export async function createWorkerCube({
  workerUrl = new URL('./worker.mjs', import.meta.url),
  wasmUrl,
  initializationTimeoutMs = 30000,
} = {}) {
  if (!(workerUrl instanceof URL) || (wasmUrl !== undefined && !(wasmUrl instanceof URL))) throw new CubeError('invalid_parameter', 'Worker and Wasm asset locations must be explicit URLs.');
  timeout(initializationTimeoutMs);
  let worker;
  let generation = 0;
  let nextId = 0;
  let pending;
  let readiness;
  let busy = false;
  let identity;
  let adapters;
  let executors;
  let limits;

  function stop(code = 'worker_terminated') {
    generation++;
    worker?.terminate();
    worker = undefined;
    readiness = undefined;
    const request = pending;
    pending = undefined;
    request?.reject(new CubeError(code, code === 'worker_terminated'
      ? 'The dedicated worker was terminated; no result or cooperative cancellation is confirmed.'
      : code === 'worker_timeout'
        ? 'The deadline expired and the dedicated worker was terminated; no result or Core cancellation is confirmed.'
        : 'The dedicated worker failed; retry in a fresh worker.'));
  }

  function exchange(id, message) {
    return new Promise((resolve, reject) => {
      pending = { id, resolve, reject };
      try { worker.postMessage(message); }
      catch { pending = undefined; reject(new CubeError('invalid_input', 'Worker request could not be copied.')); }
    });
  }

  function start() {
    if (readiness) return readiness;
    const currentGeneration = ++generation;
    try { worker = new Worker(workerUrl, { type: 'module', name: 'Cube SDK' }); }
    catch { return Promise.reject(new CubeError('initialization_failed', 'Could not create the SDK module worker; check its URL and browser policy.')); }
    worker.onmessage = ({ data }) => {
      if (generation !== currentGeneration || !pending || data.id !== pending.id) return;
      const request = pending;
      pending = undefined;
      if (data.error) {
        if (data.error.code === 'runtime_failure') stop('worker_failed');
        request.reject(new CubeError(data.error.code, data.error.message, data.error));
      } else request.resolve(data.value);
    };
    worker.onerror = event => { event.preventDefault(); if (generation === currentGeneration) stop('worker_failed'); };
    worker.onmessageerror = () => { if (generation === currentGeneration) stop('worker_failed'); };
    const id = ++nextId;
    const deadline = setTimeout(() => stop('worker_timeout'), initializationTimeoutMs);
    readiness = exchange(id, { id, method: 'initialize', wasmUrl: wasmUrl?.href })
      .then(value => {
        if (value.build.wasmSha256 !== build.wasmSha256 || value.build.sourceSha256 !== build.sourceSha256 || value.build.metadataSha256 !== build.metadataSha256) {
          throw new CubeError('incompatible_runtime', 'Worker and SDK build identities differ.');
        }
        identity = immutable(value.build);
        adapters = immutable(value.pipelineAdapters);
        executors = immutable(value.executors);
        limits = immutable(value.limits);
      })
      .catch(error => { stop(); throw error; })
      .finally(() => clearTimeout(deadline));
    return readiness;
  }

  async function invoke(method, args, { signal, context = null, timeoutMs = 30000 } = {}) {
    if (busy) throw new CubeError('resource_limit', 'One invocation may be pending in this dedicated worker.');
    timeout(timeoutMs);
    if (signal && (typeof signal.addEventListener !== 'function' || typeof signal.removeEventListener !== 'function')) throw new CubeError('invalid_parameter', 'Use an AbortSignal for worker termination.');
    if (signal?.aborted) throw new CubeError('not_started', 'The invocation was aborted before dispatch.');
    snapshotBudget({ args, context });
    let snapshot;
    try { snapshot = ownedCopy({ args, context }); }
    catch { throw new CubeError('invalid_input', 'Use copyable invocation values and context.'); }
    busy = true;
    const abort = () => stop();
    signal?.addEventListener('abort', abort, { once: true });
    const deadline = setTimeout(() => stop('worker_timeout'), timeoutMs);
    try {
      await start();
      if (signal?.aborted) throw new CubeError('not_started', 'The invocation was aborted before dispatch.');
      const id = ++nextId;
      const dispatchedGeneration = generation;
      const value = await exchange(id, { id, method, args: snapshot.args });
      return { requestId: id, generation: dispatchedGeneration, context: snapshot.context, value };
    } finally {
      signal?.removeEventListener('abort', abort);
      clearTimeout(deadline);
      busy = false;
    }
  }

  await start();
  return Object.freeze({
    get build() { return identity; },
    get pipelineAdapters() { return adapters; },
    get executors() { return executors; },
    get limits() { return limits; },
    execute: (request, options) => invoke('execute', [request], options),
    catalogJson: options => invoke('catalogJson', [], options),
    recipeJson: (definition, options) => invoke('recipeJson', [definition], options),
    validateRecipe: (definition, options) => invoke('validateRecipe', [definition], options),
    runRecipe: (definition, bindings = {}, execution = {}, options) => invoke('runRecipe', [definition, bindings, execution], options),
    terminate: () => stop(),
  });
}
