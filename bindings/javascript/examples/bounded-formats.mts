import { createCube } from '@cubearc/cube';

const cube = await createCube();
const source = 'BANANA';
const data = new TextEncoder().encode(source);
const encoded = cube.execute({ operation: 'binary.bwt.encode', version: 1, inputs: { data } });
const decoded = cube.execute({
  operation: 'binary.bwt.decode', version: 1, inputs: { data: encoded.output },
  parameters: { primary_index: encoded.primary_index },
});
if (new TextDecoder().decode(decoded.output) !== source) throw new Error('BWT round trip changed the source.');
