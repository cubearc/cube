import { createCube } from '@cubearc/cube';

const cube = await createCube();
const original = (1n << 128n) - 1n;
const { report } = cube.execute({
  operation: 'number.gray',
  version: 1,
  inputs: { request: { direction: 'encode', width: 128, value: original.toString(), binary: true } },
});
const reflected = BigInt(report.value);
if (reflected !== 1n << 127n || report.binary !== '1' + '0'.repeat(127)) {
  throw new Error('Unexpected 128-bit Gray code');
}
console.log(reflected.toString());
