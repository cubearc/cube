import { createCube } from '@cubearc/cube';

const cube = await createCube();
const formula = { variables: [1, 2], clauses: [[1, 2], [-1, 2], [1, -2], [-1, -2]] };
const solve = cube.execute({
  operation: 'logic.cdcl',
  version: 1,
  inputs: { request: { action: 'solve', formula, proof: true } },
}).report;
if (solve.result.status !== 'unsat' || !solve.result.proof) {
  throw new Error('Expected an unsatisfiability proof');
}
const proof = cube.execute({
  operation: 'logic.cdcl',
  version: 1,
  inputs: { request: { action: 'verify_proof', formula, clauses: solve.result.proof } },
}).report;
if (proof.result.status !== 'verified') {
  throw new Error('The proof did not pass RUP verification');
}
console.log(proof.profile);
