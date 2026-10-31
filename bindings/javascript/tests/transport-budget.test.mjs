import assert from 'node:assert/strict';
import test from 'node:test';
import { stringifyExact } from '../dist/transport.mjs';

test('serialization stops at its byte budget before visiting later values or encoding oversize bytes', () => {
  const values = ['x'.repeat(64)];
  Object.defineProperty(values, 1, { enumerable: true, get() { throw new Error('visited after exhaustion'); } });
  assert.throws(() => stringifyExact(values, () => '', { maxBytes: 32 }), { code: 'resource_limit' });
  let encoded = false;
  assert.throws(() => stringifyExact(new Uint8Array(64), () => { encoded = true; return ''; }, { maxBytes: 32 }), { code: 'resource_limit' });
  assert.equal(encoded, false);
});
