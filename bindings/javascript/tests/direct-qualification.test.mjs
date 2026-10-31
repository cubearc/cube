import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import init, * as wasm from '../dist/wasm/cube_wasm.js';

await init({ module_or_path: await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url)) });
const fixtures = JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8'));

test('all module-owned direct executors retain canonical named inputs, outputs and versions in Wasm', () => {
  const executors = JSON.parse(wasm.executors_json());
  assert.ok(executors.some(row => row.operation === 'image.pixels.export'));
  assert.ok(executors.some(row => row.operation === 'text.concat'));
  assert.ok(!executors.some(row => row.operation === 'file.info.inspect'));
  for (const executor of executors) {
    const fixture = fixtures.find(row => row.operation === executor.operation);
    assert.ok(fixture, executor.operation);
    const request = { operation: executor.operation, version: fixture.operation_version, inputs: fixture.inputs, parameters: fixture.parameters };
    let actual;
    assert.doesNotThrow(() => { actual = JSON.parse(wasm.execute_json(JSON.stringify(request))); }, executor.operation);
    assert.deepEqual(actual, fixture.outputs, executor.operation);
    assert.deepEqual(Object.keys(executor.outputs).sort(), Object.keys(actual).sort());
  }
});
