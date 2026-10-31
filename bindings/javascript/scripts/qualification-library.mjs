// Repository qualification tooling over real runtimes and reviewed synthetic fixtures.
// This is neither an operation registry nor part of the published runtime API.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import { pipelineCases } from '../tests/support/pipeline-cases.mjs';

const sha256 = value => createHash('sha256').update(value).digest('hex');
function ordered(value) {
  if (Array.isArray(value)) return value.map(ordered);
  if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(key => [key, ordered(value[key])]));
  return value;
}
const digest = value => sha256(JSON.stringify(ordered(value)));

function indexed(rows, label) {
  assert.ok(Array.isArray(rows), `Missing ${label}; rebuild the matching runtime.`);
  const result = new Map(rows.map(row => [row.operation, row]));
  assert.equal(result.size, rows.length, `Duplicate ${label} identity.`);
  return result;
}

function verifyCatalog(snapshot, canonicalJson, fixtures, build) {
  assert.equal(snapshot.json.trim(), canonicalJson.trim(), 'Actual runtime catalog differs from the canonical wire.');
  assert.equal(snapshot.sha256, sha256(snapshot.json.trim()), 'Runtime catalog digest mismatch.');
  assert.equal(snapshot.sha256, build.catalogSha256, 'SDK/catalog identity drift.');
  const catalog = JSON.parse(snapshot.json);
  const contracts = indexed(catalog.operations, 'catalog');
  const identities = indexed(snapshot.contracts, 'runtime contract hashes');
  const answers = indexed(fixtures, 'reviewed fixtures');
  assert.deepEqual([...answers.keys()].sort(), [...contracts.keys()].sort(), 'Fixture/catalog ID drift.');
  assert.deepEqual([...identities.keys()].sort(), [...contracts.keys()].sort(), 'Runtime contract ID drift.');
  for (const [operation, contract] of contracts) {
    const identity = identities.get(operation); const fixture = answers.get(operation);
    assert.equal(identity.operation_version, contract.operation_version, `Runtime version drift: ${operation}`);
    assert.equal(identity.contract_sha256, fixture.contract_sha256, `Review the changed contract: ${operation}`);
    assert.equal(contract.operation_version, fixture.operation_version, `Fixture version drift: ${operation}`);
  }
  return { catalog, contracts, answers, identities };
}

function verifyCapabilities(entries, contracts, path) {
  const result = indexed(entries, `${path} capabilities`);
  for (const entry of result.values()) {
    const contract = contracts.get(entry.operation);
    assert.ok(contract, `Unknown ${path} capability: ${entry.operation}`);
    assert.equal(entry.version, path === 'direct' ? contract.operation_version : contract.pipeline_adapter, `Capability version drift: ${entry.operation}`);
  }
  return result;
}

function checkResult(actual, expected, operation, path) {
  if (!isDeepStrictEqual(actual, expected)) throw new Error(`Actual Wasm known-answer mismatch: ${operation} (${path}).`);
}

function evidence(contract, version, request, actual, expected, fixture, path) {
  checkResult(actual, expected, contract.operation, path);
  const retain = !contract.policy.private_output;
  return {
    status: 'passed', execution: 'actual-wasm', version,
    caseSha256: digest(request), reviewedFixtureSha256: digest(fixture),
    actualOutputSha256: digest(actual), expectedOutputSha256: digest(expected),
    outputPorts: path === 'direct' ? Object.keys(actual).sort() : ['output'],
    outputRetained: retain,
    ...(retain ? { actualOutputs: actual } : { redaction: 'Core private-output policy; exact output equality was checked before hashing.' }),
  };
}

async function qualify(contract, fixture, binding, adapter, pipelineCase, driver) {
  const result = {};
  if (binding) {
    const request = { operation: contract.operation, version: binding.version, inputs: fixture.inputs, parameters: fixture.parameters };
    const actual = await driver.executeDirect(request, binding);
    result.direct = evidence(contract, binding.version, request, actual, fixture.outputs, fixture, 'direct');
  }
  if (adapter) {
    const execution = await driver.runRecipe(pipelineCase.definition);
    checkResult(execution.intermediates['0'], pipelineCase.expected, contract.operation, 'pipeline capture');
    result.pipeline = { ...evidence(contract, adapter.version, pipelineCase.definition, execution.output, pipelineCase.expected, fixture, 'pipeline'),
      capturedIntermediate: true, fixtureProjection: 'reviewed pipeline-cases-v1' };
  }
  return result;
}

export async function qualificationReport({ snapshot, canonicalJson, fixtures, build, executors, adapters, limits, runtime, driver }) {
  const { catalog, contracts, answers, identities } = verifyCatalog(snapshot, canonicalJson, fixtures, build);
  const direct = verifyCapabilities(executors, contracts, 'direct');
  const pipelines = verifyCapabilities(adapters, contracts, 'pipeline');
  const cases = indexed(pipelineCases(adapters, fixtures), 'pipeline cases');
  const operations = [];
  for (const contract of catalog.operations) {
    const binding = direct.get(contract.operation); const adapter = pipelines.get(contract.operation);
    const covered = await qualify(contract, answers.get(contract.operation), binding, adapter, cases.get(contract.operation), driver);
    const supported = Boolean(binding || adapter);
    operations.push({
      operation: contract.operation, operationVersion: contract.operation_version,
      contractSha256: identities.get(contract.operation).contract_sha256,
      direct: binding ?? null, pipeline: adapter ?? null, evidence: covered,
      assessment: supported ? 'qualified-in-memory' : 'requires-binding-and-qualification',
      reason: supported ? 'The advertised paths passed actual Wasm known answers in this recorded runtime; input compatibility still requires Core prevalidation.'
        : `No installed typed binding is registered for its canonical named ports (${contract.inputs.map(port => port.semantic_type).join(', ')}). Native conformance and catalog presence do not qualify browser execution.`,
    });
  }
  return {
    reportVersion: 2, status: 'passed',
    scope: 'Actual synthetic known-answer execution in the recorded runtime only; no other engine/host or production release is inferred.',
    runtime, build, limits, fixtureSetSha256: digest(fixtures),
    retention: 'No inputs or parameters retained. Results are retained only when the canonical Core output policy permits; otherwise equality and exact digests are recorded.',
    catalog: { formatVersion: catalog.format_version, engineVersion: catalog.engine_version, sha256: snapshot.sha256, operationCount: operations.length },
    coverage: { direct: direct.size, pipeline: pipelines.size }, operations,
  };
}

/** Test transport only: actual operation execution stays in the installed SDK. */
export function sdkDriver(cube) {
  const toWire = value => value.type === 'bytes' ? { type: 'bytes', base64: Buffer.from(value.data).toString('base64') } : value;
  return {
    executeDirect(request, binding) {
      const inputs = Object.fromEntries(Object.entries(request.inputs).map(([name, value]) => [name,
        binding.inputs[name] === 'bytes' ? Uint8Array.from(Buffer.from(value.base64, 'base64'))
          : binding.inputs[name] === 'byte_array' ? Uint8Array.from(value) : value]));
      const actual = cube.execute({ ...request, inputs });
      return Object.fromEntries(Object.entries(actual).map(([name, value]) => [name,
        binding.outputs[name] === 'bytes' ? { base64: Buffer.from(value).toString('base64') }
          : binding.outputs[name] === 'byte_array' ? Array.from(value) : value]));
    },
    runRecipe(definition) {
      const run = cube.runRecipe(definition, {}, { captures: [0] });
      return { output: toWire(run.output), intermediates: { '0': toWire(run.intermediates['0']) } };
    },
  };
}
