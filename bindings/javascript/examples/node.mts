import { createCube, type RecipeDefinition } from '@cubearc/cube';

const cube = await createCube();
const bytes: Uint8Array = cube.execute({
  operation: 'encoding.base64.decode', version: 1, inputs: { input: 'AP8=' },
}).output;
if (bytes.length !== 2 || bytes[0] !== 0 || bytes[1] !== 255) throw new Error('Exact byte example failed.');

const definition: RecipeDefinition = {
  format_version: 1, engine_api: 1,
  input: { source: 'slot', name: 'message', type: 'text' },
  output: { type: 'text' },
  steps: [
    { operation: 'encoding.base64.decode', version: 1 },
    { operation: 'text.utf8.decode', version: 1 },
  ],
};
const result = cube.runRecipe(definition, { message: { type: 'text', data: 'Q2Fmw6k=' } }, { captures: [0] });
if (result.output.type !== 'text' || result.output.data !== 'Café') throw new Error('Unicode recipe example failed.');
const first = result.intermediates['0'];
if (first?.type !== 'bytes' || first.data.length !== 5 || first.data[3] !== 195 || first.data[4] !== 169) throw new Error('Typed intermediate example failed.');
console.log('TypeScript: Café; exact bytes', Array.from(bytes));

const primeGroups = cube.execute({
  operation: 'cipher.prime_multiplication.decode', version: 1, inputs: { products: ['6'] },
  parameters: { profile: 'prime_multiset_v1', index: { alphabet: 'AB', first_index: 1, max_prime: 3 }, max_factor_checks: 10 },
}).groups;
if (primeGroups[0].product !== '6' || primeGroups[0].canonical_text !== 'AB') throw new Error('Prime multiset mismatch');
console.log(primeGroups[0]);
