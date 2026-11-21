import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import init, * as wasm from '../dist/wasm/cube_wasm.js';
import {verifyFixtureResult} from '../scripts/fixture-verification.mjs';

await init({ module_or_path: await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm', import.meta.url)) });
const fixtures = JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8'));

test('all module-owned direct executors retain canonical named inputs, outputs and versions in Wasm', async () => {
  const executors = JSON.parse(wasm.executors_json());
  assert.ok(executors.some(row => row.operation === 'image.pixels.export'));
  assert.ok(executors.some(row => row.operation === 'text.concat'));
  assert.equal(executors.length, fixtures.length);
  assert.ok(executors.some(row => row.operation === 'file.info.inspect'));
  const failures = [];
  const contracts=new Map(JSON.parse(JSON.parse(wasm.catalog_snapshot_json()).json).operations.map(c=>[c.operation,c]));
  const execute=request=>JSON.parse(wasm.execute_json(JSON.stringify(request)));
  for (const executor of executors) {
    const fixture = fixtures.find(row => row.operation === executor.operation);
    assert.ok(fixture, executor.operation);
    const request = { operation: executor.operation, version: fixture.operation_version, inputs: fixture.inputs, parameters: fixture.parameters };
    try {
      const contract = contracts.get(fixture.operation);
      const actual = execute(request);
      await verifyFixtureResult({ fixture, actual, contract, execute });
      for (const direction of ['inputs', 'outputs']) {
        assert.deepEqual(Object.keys(executor[direction]).sort(), contract[direction].map(port => port.name).sort());
      }
      const expectedOutputs = contract.outputs
        .filter(port => port.required || Object.hasOwn(fixture.outputs, port.name))
        .map(port => port.name)
        .sort();
      assert.deepEqual(Object.keys(actual).sort(), expectedOutputs);
    } catch (error) {
      failures.push(`${executor.operation}: ${String(error.message ?? error).slice(0, 1600)}`);
    }
  }
  assert.deepEqual(failures, []);
});
