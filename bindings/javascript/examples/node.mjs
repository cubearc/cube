import assert from 'node:assert/strict';
import { createCube } from '@cubearc/cube';

const cube = await createCube();
const decoded = cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: 'AP8=' } });
assert.deepEqual(decoded.output, Uint8Array.of(0, 255));
const joined = cube.execute({ operation: 'text.concat', version: 1, inputs: { left: 'Cube', right: 'Arc' }, parameters: { separator: '' } });
assert.equal(joined.output, 'CubeArc');
console.log(joined.output, Array.from(decoded.output));

const numbered = cube.execute({
  operation: 'cipher.numbered_key.encode', version: 1, inputs: { text: 'A ee iii' },
  parameters: { alphabet: 'abcdefghijklmnopqrstuvwxyz', key: 'I like ciphers.', preparation: 'ascii_lowercase_letters', rotation: 18, origin: '0', width: 2, choice: 'round_robin' },
});
assert.deepEqual(numbered.tokens, ['23', '15', '20', '11', '13', '17']);
console.log(numbered.tokens.join(' '));

const readings = cube.execute({
  operation: 'cipher.key_phrase.decode', version: 1, inputs: { text: 'I' },
  parameters: { phrase: 'GIVEMELIBERTYORGIVEMEDEATH' },
});
assert.equal(readings.text, null);
assert.deepEqual(readings.records[0].alternatives, ['B', 'H', 'Q']);
console.log(readings.records[0].alternatives);

const syllables = cube.execute({
  operation: 'symbol.solresol.encode', version: 1, inputs: { terms: ['non', 'oui', 'à'] },
  parameters: { profile: 'gajewski_1902_monosyllables_v1' },
});
assert.deepEqual(syllables.tokens, ['do', 'si', 'fa']);
console.log(syllables.tokens);

const primeGroups = cube.execute({
  operation: 'cipher.prime_multiplication.decode', version: 1, inputs: { products: ['6'] },
  parameters: { profile: 'prime_multiset_v1', index: { alphabet: 'AB', first_index: 1, max_prime: 3 }, max_factor_checks: 10 },
}).groups;
if (primeGroups[0].product !== '6' || primeGroups[0].canonical_text !== 'AB') throw new Error('Prime multiset mismatch');
console.log(primeGroups[0]);
