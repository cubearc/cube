import { createCube } from '@cubearc/cube';

const cube = await createCube();
const input = Uint8Array.of(0x41, 0, 0xff);
const { output: encoded } = cube.execute({
  operation: 'encoding.url_component.encode',
  version: 1,
  inputs: { input },
});
console.log(encoded); // A%00%FF

const { output: compressed } = cube.execute({
  operation: 'binary.compress.encode',
  version: 1,
  inputs: { input },
  parameters: { profile: 'zlib' },
});
const { output } = cube.execute({
  operation: 'binary.compress.decode',
  version: 1,
  inputs: { input: compressed },
  parameters: { profile: 'zlib' },
});
console.log(Array.from(output)); // [65, 0, 255]
