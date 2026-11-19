import { createCube } from '@cubearc/cube';

const cube = await createCube();
const { report } = cube.execute({
  operation: 'text.graphemes',
  version: 1,
  inputs: { request: { action: 'reverse', text: 'á🧑‍💻b' } },
});
if (report.action !== 'reverse' || report.text !== 'b🧑‍💻á') {
  throw new Error('Unexpected grapheme reversal');
}
console.log(report.text);
