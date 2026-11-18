import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { createCube } from '../dist/node.mjs';

const cube = await createCube();
const bytes = (...values) => Uint8Array.from(values);
const text = value => new TextEncoder().encode(value);
const run = (operation, input, parameters) => cube.execute({
  operation, version: 1, inputs: { input }, parameters,
}).output;

test('byte and Unicode profiles use exact named Wasm ports', () => {
  assert.equal(run('encoding.binary.encode', bytes(0, 255)), '00000000 11111111');
  assert.deepEqual(run('encoding.url_component.decode', '%00%Ff%2B'), bytes(0, 255, 43));
  assert.deepEqual(run('binary.reverse_bits', bytes(1, 2, 128, 0), { unit_bytes: 2 }), bytes(64, 128, 0, 1));
  assert.equal(run('hash.sha3_256', text('abc')), '3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532');
  assert.equal(run('hash.crc8_smbus', text('123456789')), 'f4');
  assert.equal(run('text.json_escape.encode', '🧊'), '\\uD83E\\uDDCA');
  assert.equal(run('text.html_entity.decode', '&notin; &#128;'), '∉ €');
  assert.equal(run('text.utf16.decode', bytes(0xfe, 0xff, 0, 65), { byte_order: 'be', bom: 'strip' }), 'A');
  assert.throws(() => run('encoding.base92.decode', '}}'), { code: 'invalid_input' });
  assert.throws(() => run('encoding.base62.encode', new Uint8Array(4097)), { code: 'resource_limit' });
});

test('all six compression profiles execute through the packaged Wasm', () => {
  const input = text('Cube\0\n'.repeat(19));
  for (const profile of ['deflate', 'zlib', 'gzip', 'brotli', 'zstd', 'lzma_alone']) {
    const encoded = run('binary.compress.encode', input, { profile });
    assert.deepEqual(run('binary.compress.decode', encoded, { profile }), input, profile);
    assert.throws(() => run('binary.compress.decode', bytes(...encoded, 0), { profile }), { code: 'invalid_input' });
  }
});

test('private KDF and JPEG evidence use separate typed values', async () => {
  const argon = cube.execute({
    operation: 'cipher.kdf.argon2id', version: 1,
    inputs: { password: text('password'), salt: text('12345678') },
    parameters: { memory_kib: 32, iterations: 2, lanes: 1, output_bytes: 32 },
  });
  assert.equal(Buffer.from(argon.output).toString('hex'), 'a76b1652c8a7f4fac2d0805f6982e354cb0d9d580cc5553a197f4d79e599e534');
  const source = await readFile(new URL('../../../crates/cube-core/src/image/jpeg_coefficients/fixture.jpg', import.meta.url));
  const extracted = cube.execute({
    operation: 'image.jpeg.coefficients.extract', version: 1,
    inputs: { input: new Uint8Array(source) },
    parameters: { profile: 'baseline_ac_parity', component_id: 1, start_bit: 3, max_bits: 19 },
  });
  assert.deepEqual(extracted.bits, bytes(0xa4, 0x48, 0x60));
  assert.equal(extracted.report.candidate_bits, 193);
});
