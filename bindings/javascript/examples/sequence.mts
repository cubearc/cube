import { createCube } from '@cubearc/cube';

const cube = await createCube();
const { report } = cube.execute({
  operation: 'sequence.edit_distance',
  version: 1,
  inputs: {
    request: {
      operands: { unit: 'scalars', left: 'CA', right: 'ABC' },
      method: 'dl',
    },
  },
});
if (report.distance !== 2) throw new Error('Unexpected edit distance');
console.log(report.distance);
