import { readFile } from 'node:fs/promises';
import { createCube as initialize } from './browser.mjs';
import { CubeError } from './errors.mjs';
export { CubeError } from './errors.mjs';

const fileInitializations = new Map();
export function createCube(options = {}) {
  if (!options || typeof options !== 'object' || Object.keys(options).some(key => key !== 'wasm')) return Promise.reject(new CubeError('invalid_parameter', 'Use an optional Wasm source configuration.'));
  const source = options.wasm ?? new URL('./wasm/cube_wasm_bg.wasm', import.meta.url);
  if (!(source instanceof URL) || source.protocol !== 'file:') return initialize({ wasm: source });
  if (!fileInitializations.has(source.href)) {
    const pending = readFile(source)
      .catch(() => { throw new CubeError('initialization_failed', 'Could not read the configured Cube Wasm asset.'); })
      .then(wasm => initialize({ wasm }))
      .catch(error => { fileInitializations.delete(source.href); throw error; });
    fileInitializations.set(source.href, pending);
  }
  return fileInitializations.get(source.href);
}
