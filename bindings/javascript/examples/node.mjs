import assert from 'node:assert/strict';
import { createCube } from '@cubearc/cube';

const cube = await createCube();
const decoded = cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: 'AP8=' } });
assert.deepEqual(decoded.output, Uint8Array.of(0, 255));
const joined = cube.execute({ operation: 'text.concat', version: 1, inputs: { left: 'Cube', right: 'Arc' }, parameters: { separator: '' } });
assert.equal(joined.output, 'CubeArc');
console.log(joined.output, Array.from(decoded.output));
