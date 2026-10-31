import { CubeError } from './errors.mjs';
import { build } from './build-info.mjs';

const MAX_ASSET_BYTES = 64 * 1024 * 1024;

async function fetchedBytes(url) {
  let response;
  try { response = await fetch(url, { cache: 'no-cache' }); }
  catch { throw new CubeError('initialization_failed', 'Could not fetch the Cube Wasm asset; check its URL and network access.'); }
  if (!response.ok || !response.body) throw new CubeError('initialization_failed', 'Cube Wasm asset is missing or has an unsuccessful response.');
  const reader = response.body.getReader();
  const chunks = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > MAX_ASSET_BYTES) throw new CubeError('resource_limit', 'Wasm asset exceeds 64 MiB.');
      chunks.push(value);
    }
  } catch (error) {
    await reader.cancel().catch(() => {});
    if (error instanceof CubeError) throw error;
    throw new CubeError('initialization_failed', 'The Cube Wasm asset download was interrupted.');
  } finally { reader.releaseLock(); }
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
  return bytes;
}

export async function verifiedBytes(source) {
  const bytes = source instanceof Uint8Array ? source : await fetchedBytes(source);
  if (bytes.byteLength > MAX_ASSET_BYTES) throw new CubeError('resource_limit', 'Wasm asset exceeds 64 MiB.');
  if (!globalThis.crypto?.subtle) throw new CubeError('initialization_failed', 'Cube requires Web Crypto in a secure browser context.');
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
  const hex = Array.from(digest, value => value.toString(16).padStart(2, '0')).join('');
  if (hex !== build.wasmSha256) throw new CubeError('incompatible_runtime', 'Wasm bytes do not match this SDK build; restore the matching package assets.');
  return bytes;
}
