import init, * as wasm from './wasm/cube_wasm.js';
import { CubeError, coreCall, requireText } from './errors.mjs';
import { boundedDocument, captureIndices, executionFromWire, immutable, stringifyExact } from './transport.mjs';
import { verifiedBytes } from './loading.mjs';
import { build } from './build-info.mjs';
export { CubeError } from './errors.mjs';

let initialization;
let initializationSource;
let executors;
let limits;

function directExecutors() {
  executors ??= immutable(coreCall(() => JSON.parse(wasm.executors_json())));
  return executors;
}

function runtimeLimits() {
  limits ??= immutable(coreCall(() => JSON.parse(wasm.runtime_limits_json())));
  return limits;
}

function document(value) {
  return typeof value === 'string'
    ? boundedDocument(value)
    : stringifyExact(value, bytes => coreCall(() => wasm.encode_base64(bytes)));
}

function execute(request) {
  if (!request || typeof request !== 'object') throw new CubeError('invalid_input', 'Provide a named invocation.');
  const { operation, version, inputs, parameters = {} } = request;
  const executor = directExecutors().find(value => value.operation === operation);
  if (!executor) {
    throw new CubeError('unsupported_operation', 'No direct binding is available; inspect pipeline support separately.');
  }
  if (version !== executor.version) throw new CubeError('incompatible_version', 'Use the installed executor contract version.', { operation });
  if (!inputs || typeof inputs !== 'object' || Array.isArray(inputs) ||
      !parameters || typeof parameters !== 'object' || Array.isArray(parameters) ||
      Object.keys(request).some(key => !['operation', 'version', 'inputs', 'parameters'].includes(key))) {
    throw new CubeError('invalid_input', 'Use exact named inputs and parameters.', { operation });
  }
  const bounds = runtimeLimits().direct;
  const wireInputs = Object.fromEntries(Object.entries(inputs).map(([name, value]) => {
    const representation = executor.inputs[name];
    if (value instanceof Uint8Array && value.byteLength > bounds.inputBytes) throw new CubeError('resource_limit', 'Named input exceeds the direct byte budget.', { operation });
    if (representation === 'bytes') {
      if (!(value instanceof Uint8Array)) throw new CubeError('invalid_input', 'Expected Uint8Array for the named byte input.', { operation });
      return [name, { base64: coreCall(() => wasm.encode_base64(value)) }];
    }
    if (representation === 'byte_array' && value instanceof Uint8Array) return [name, Array.from(value)];
    if (representation === 'text') requireText(value);
    return [name, value];
  }));
  const invocation = stringifyExact({ operation, version, inputs: wireInputs, parameters },
    bytes => coreCall(() => wasm.encode_base64(bytes)), { maxBytes: bounds.documentBytes, maxDepth: bounds.documentDepth, maxNodes: bounds.documentNodes });
  const output = coreCall(() => JSON.parse(wasm.execute_json(invocation)));
  return Object.fromEntries(Object.entries(output).map(([name, value]) => [name,
    executor.outputs[name] === 'bytes' ? coreCall(() => wasm.decode_base64(value.base64))
      : executor.outputs[name] === 'byte_array' ? Uint8Array.from(value) : value,
  ]));
}

function api() {
  let catalog;
  const pipelineAdapters = immutable(coreCall(() => JSON.parse(wasm.pipeline_adapters_json())));
  return Object.freeze({
    build: Object.freeze(build),
    limits: runtimeLimits(),
    execute,
    get executors() { return directExecutors(); },
    recipeJson(definition) { return document(definition); },
    pipelineAdapters,
    catalogJson() {
      if (catalog === undefined) {
        const snapshot = coreCall(() => JSON.parse(wasm.catalog_snapshot_json()));
        if (snapshot.sha256 !== build.catalogSha256) throw new CubeError('incompatible_runtime', 'The runtime catalog differs from this SDK build.');
        catalog = snapshot.json;
      }
      return catalog;
    },
    validateRecipe(definition) { return coreCall(() => JSON.parse(wasm.validate_recipe_json(document(definition)))); },
    runRecipe(definition, bindings = {}, { captures = [] } = {}) {
      const indices = captureIndices(captures);
      const result = coreCall(() => JSON.parse(wasm.run_recipe_json(document(definition), document(bindings), indices)));
      return executionFromWire(result, bytes => coreCall(() => wasm.decode_base64(bytes)));
    },
  });
}

/** One Wasm instance per module realm. Failed initialization may be retried. */
export function createCube(options = {}) {
  if (!options || typeof options !== 'object' || Object.keys(options).some(key => key !== 'wasm')) return Promise.reject(new CubeError('invalid_parameter', 'Use an optional Wasm source configuration.'));
  const source = options.wasm ?? new URL('./wasm/cube_wasm_bg.wasm', import.meta.url);
  const key = source instanceof URL ? source.href : source;
  if (!(source instanceof URL) && !(source instanceof Uint8Array)) {
    return Promise.reject(new CubeError('invalid_parameter', 'Wasm source must be a URL or Uint8Array.'));
  }
  if (initialization) {
    if (initializationSource !== key) return Promise.reject(new CubeError('initialization_conflict', 'This module realm already has a different Wasm source.'));
    return initialization;
  }
  initializationSource = key;
  const snapshot = source instanceof Uint8Array ? new Uint8Array(source) : source;
  initialization = (async () => {
    const bytes = await verifiedBytes(snapshot);
    try { await init({ module_or_path: bytes }); }
    catch { throw new CubeError('initialization_failed', 'Could not load Cube Wasm; check its URL, response, MIME type and package version.'); }
    return api();
  })().catch(error => { initialization = undefined; initializationSource = undefined; throw error; });
  return initialization;
}
