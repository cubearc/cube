import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import init,* as wasm from '../dist/wasm/cube_wasm.js';
await init({module_or_path:await readFile(new URL('../dist/wasm/cube_wasm_bg.wasm',import.meta.url))});
const execute = (operation, parameters) => wasm.execute_json(JSON.stringify({operation,version:2,inputs:{source:',.',input:[255]},parameters}));
test('diagnostic interpreter bindings honor the installed SDK instruction ceiling', () => {
  assert.throws(() => execute('esolang.brainfuck.report',{max_instructions:1000001}), error => JSON.parse(error).code === 'invalid_parameter');
  assert.throws(() => execute('esolang.brainfuck.compare',{widths:[8,16],total_instructions:1000001}), error => JSON.parse(error).code === 'invalid_parameter');
});

test('traces, selected tape cells and memory failures use the portable conservative ledger', () => {
  const parameters = { max_output_bytes:1, max_trace_records:2, max_trace_bytes:96, tape_len:1, max_report_bytes:4788, max_memory_bytes:126976 };
  const report = JSON.parse(execute('esolang.brainfuck.report', parameters)).output;
  assert.equal(report.memory_bytes_upper_bound,126976);
  assert.equal(report.report_bytes_upper_bound,4788);
  assert.deepEqual(report.output,[255]);
  assert.equal(report.trace.length,2);
  assert.equal(report.tape[0].value,255);
  assert.throws(() => execute('esolang.brainfuck.report',{...parameters,max_memory_bytes:126975}), error => JSON.parse(error).code === 'resource_limit');
});
