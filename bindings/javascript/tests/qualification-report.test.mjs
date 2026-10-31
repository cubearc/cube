import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

test('qualification CLI emits real execution evidence for every advertised path', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'cube-qualified-'));
  try {
    const destination = join(directory, 'report.json');
    const child = spawnSync(process.execPath, ['scripts/qualification-report.mjs', destination], { cwd: new URL('..', import.meta.url), encoding: 'utf8' });
    assert.equal(child.status, 0, child.stderr);
    const report = JSON.parse(await readFile(destination, 'utf8'));
    assert.equal(report.reportVersion, 2, 'Inventory alone must not claim real execution qualification.');
    assert.equal(report.runtime.kind, 'node-wasm');
    const catalog = JSON.parse(await readFile(new URL('../../../contracts/v1/catalog.json', import.meta.url), 'utf8'));
    assert.deepEqual(report.operations.map(row => row.operation), catalog.operations.map(row => row.operation));
    const fixtures = JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8'));
    for (const row of report.operations) {
      const fixture = fixtures.find(fixture => fixture.operation === row.operation);
      assert.equal(row.contractSha256, fixture.contract_sha256);
      for (const path of ['direct', 'pipeline']) {
        if (!row[path]) continue;
        assert.equal(row.evidence[path].status, 'passed', `${row.operation}: ${path}`);
        assert.equal(row.evidence[path].actualOutputSha256, row.evidence[path].expectedOutputSha256);
        const contract = catalog.operations.find(contract => contract.operation === row.operation);
        assert.equal(row.evidence[path].outputRetained, !contract.policy.private_output);
        if (contract.policy.private_output) assert.equal(row.evidence[path].actualOutputs, undefined);
        else assert.ok(row.evidence[path].actualOutputs !== undefined);
      }
      if (!row.direct && !row.pipeline) { assert.ok(row.reason); assert.equal(Object.keys(row.evidence).length, 0); }
    }
  } finally { await rm(directory, { recursive: true, force: true }); }
});

async function inputs() {
  const { createCube } = await import('../dist/node.mjs');
  const wasm = await import('../dist/wasm/cube_wasm.js');
  const { sdkDriver } = await import('../scripts/qualification-library.mjs');
  const cube = await createCube();
  return {
    snapshot: JSON.parse(wasm.catalog_snapshot_json()),
    canonicalJson: await readFile(new URL('../../../contracts/v1/catalog.json', import.meta.url), 'utf8'),
    fixtures: JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8')),
    build: cube.build, executors: cube.executors, adapters: cube.pipelineAdapters, limits: cube.limits,
    runtime: { kind: 'node-wasm', version: process.version }, driver: sdkDriver(cube),
  };
}

test('report refuses a changed known answer instead of qualifying metadata or leaking values', async () => {
  const { qualificationReport } = await import('../scripts/qualification-library.mjs');
  const state = await inputs();
  const fixture = state.fixtures.find(row => row.operation === 'encoding.base64.decode');
  fixture.outputs.output = { base64: 'private-test-value' };
  await assert.rejects(qualificationReport(state), error => {
    assert.match(error.message, /Actual Wasm known-answer mismatch/);
    assert.ok(!error.message.includes('private-test-value'));
    return true;
  });
});

test('report rejects reviewed-contract, fixture and capability drift before qualification', async () => {
  const { qualificationReport } = await import('../scripts/qualification-library.mjs');
  let state = await inputs();
  state.fixtures[0].contract_sha256 = '0'.repeat(64);
  await assert.rejects(qualificationReport(state), /Review the changed contract/);
  state = await inputs(); state.fixtures.pop();
  await assert.rejects(qualificationReport(state), /Fixture\/catalog ID drift/);
  state = await inputs();
  state.executors = [...state.executors, { ...state.executors[0], operation: 'not.a.canonical.operation' }];
  await assert.rejects(qualificationReport(state), /Unknown direct capability/);
});
