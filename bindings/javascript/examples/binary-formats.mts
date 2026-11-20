import { createCube } from '@cubearc/cube';

const cube = await createCube();
const parameters = { bits: 64, byte_order: 'little', signedness: 'unsigned' } as const;
const values = [42, '18446744073709551615'];
const { output } = cube.execute({
  operation: 'binary.fixed_integer.encode',
  version: 1,
  inputs: { values },
  parameters,
});
const { result } = cube.execute({
  operation: 'binary.fixed_integer.decode',
  version: 1,
  inputs: { data: output },
  parameters,
});
if (BigInt(result.values[1]) !== (1n << 64n) - 1n) throw new Error('64-bit integer changed');

const { output: url } = cube.execute({
  operation: 'encoding.base64url.encode',
  version: 1,
  inputs: { input: Uint8Array.of(0xfb, 0xff) },
  parameters: { padding: 'forbidden' },
});
if (url !== '-_8') throw new Error('Unexpected URL-safe Base64');
console.log(result.values, url);
