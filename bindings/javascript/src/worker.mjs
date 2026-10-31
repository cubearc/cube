import { createCube, CubeError } from './browser.mjs';

let cube;
const methods = new Set(['execute', 'catalogJson', 'recipeJson', 'validateRecipe', 'runRecipe']);
self.onmessage = async ({ data }) => {
  const { id, method, args = [], wasmUrl } = data;
  try {
    if (method === 'initialize') {
      if (cube) throw new CubeError('initialization_conflict', 'This worker is already initialized.');
      cube = await createCube(wasmUrl ? { wasm: new URL(wasmUrl) } : {});
      self.postMessage({ id, value: { build: cube.build, limits: cube.limits, executors: cube.executors, pipelineAdapters: cube.pipelineAdapters } });
      return;
    }
    if (!cube || !methods.has(method) || !Array.isArray(args)) throw new CubeError('invalid_parameter', 'Invalid SDK worker invocation.');
    const value = cube[method](...args);
    self.postMessage({ id, value });
  } catch (failure) {
    const error = failure instanceof CubeError ? failure : new CubeError('runtime_failure', 'The worker invocation failed.');
    self.postMessage({ id, error: {
      code: error.code, message: error.message, operation: error.operation,
      parameter: error.parameter, step: error.step, offset: error.offset, reason: error.reason,
    } });
  }
};
