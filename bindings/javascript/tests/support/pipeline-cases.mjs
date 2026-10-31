// Test-only projection of reviewed Core known answers, never a runtime dispatcher.
import assert from 'node:assert/strict';

function tagged(value) {
  return typeof value === 'string'
    ? { type: 'text', data: value }
    : { type: 'bytes', base64: value.base64 };
}

function parametersFor(fixture) {
  const parameters = structuredClone(fixture.parameters);
  if (Array.isArray(parameters.key)) parameters.key = { base64: Buffer.from(parameters.key).toString('base64') };
  for (const field of ['indices', 'exclude']) {
    if (parameters[field]) parameters[field] = { unsigned_list: parameters[field] };
  }
  if (parameters.overrides) parameters.overrides = { text_list: parameters.overrides };
  // The pipeline uses signed i64 count parameters; direct rotations use u64.
  // Seven gives the reviewed fixture's same rotation of eight-bit units.
  if (fixture.operation.startsWith('binary.rotate_')) parameters.count = 7;
  if (typeof parameters.offset === 'string') {
    assert.ok(Number.isSafeInteger(Number(parameters.offset)));
    parameters.offset = Number(parameters.offset);
  }
  return parameters;
}

function expectedFor(fixture) {
  const expected = fixture.outputs.output;
  if (fixture.operation === 'file.signature.detect') {
    return { type: 'forensic_report', data: { kind: 'signatures', matches: expected } };
  }
  if (fixture.operation === 'file.hex.inspect') {
    return {
      type: 'forensic_report', data: { kind: 'hex', rows: expected.map(row => ({ ...row, bytes: Buffer.from(row.hex, 'hex').toString('base64') })) },
    };
  }
  if (fixture.operation === 'file.strings.extract') {
    return { type: 'forensic_report', data: { kind: 'strings', ...expected } };
  }
  return tagged(expected);
}

export function pipelineCases(adapters, fixtures) {
  return adapters.map(adapter => {
    const fixture = fixtures.find(row => row.operation === adapter.operation);
    assert.ok(fixture, `missing fixture: ${adapter.operation}`);
    return {
      operation: adapter.operation,
      definition: {
        format_version: 1, engine_api: 1,
        input: { source: 'inline', value: tagged(fixture.inputs.input) },
        output: { type: adapter.output_type },
        steps: [{ operation: adapter.operation, version: adapter.version, parameters: parametersFor(fixture) }],
      },
      expected: expectedFor(fixture),
    };
  });
}
