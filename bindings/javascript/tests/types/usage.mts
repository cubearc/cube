import { createCube, type RecipeDefinition } from '../../dist/node.mjs';

const cube = await createCube();
const numbered = cube.execute({operation:'cipher.numbered_key.encode',version:1,inputs:{text:'AAA'},parameters:{alphabet:'AB',key:'ABA',preparation:'exact',rotation:0,origin:'0',width:1,choice:'round_robin'}});
const numberedTokens:readonly string[]=numbered.tokens;
const headlineRows:readonly string[]=cube.execute({operation:'cipher.headlines.encode',version:1,inputs:{headlines:['','','','','']},parameters:{profile:'aca_base',hat:'APOTHECARY',key:'CHEMIST',setting:'DRUGS',punctuation:'preserve'}}).headlines;
void numberedTokens; void headlineRows;
// @ts-expect-error Numbered Key origin is exact decimal text, never a float.
cube.execute({operation:'cipher.numbered_key.encode',version:1,inputs:{text:'A'},parameters:{alphabet:'AB',key:'ABA',preparation:'exact',rotation:0,origin:0,width:1,choice:'first'}});
// @ts-expect-error Headlines requires the explicit qualified profile.
cube.execute({operation:'cipher.headlines.encode',version:1,inputs:{headlines:[]},parameters:{hat:'A',key:'B',setting:'CDEFG',punctuation:'preserve'}});
const table: string = cube.execute({operation:'cipher.cadenus.encode',version:1,inputs:{text:'A'},parameters:{keyword:'EASY',padding:'X'}}).text;
const cm: string = cube.execute({operation:'cipher.cm_bifid.encode',version:1,inputs:{text:'A'},parameters:{input_square:'ABCDEFGHIKLMNOPQRSTUVWXYZ',output_square:'ABCDEFGHIKLMNOPQRSTUVWXYZ',period:1}}).text;
const vic: string = cube.execute({operation:'cipher.vic.encode',version:1,inputs:{text:'A'},parameters:{phrase:'ALLTHEPEOPLEAREDEADB',mnemonic:'ASINTOER',date_digits:'391752',personal_number:15,message_indicator:'60115',split_at:0,padding:''}}).text;
void table; void cm; void vic;
// @ts-expect-error Both independent squares are required.
cube.execute({operation:'cipher.cm_bifid.encode',version:1,inputs:{text:'A'},parameters:{input_square:'ABCDEFGHIKLMNOPQRSTUVWXYZ',period:1}});
// @ts-expect-error Key convention and readout are explicit, not optional guesses.
cube.execute({operation:'cipher.nihilist_transposition.encode',version:1,inputs:{text:'A'},parameters:{key:[1]}});
// @ts-expect-error VIC never supplies a missing personal number or indicator.
cube.execute({operation:'cipher.vic.encode',version:1,inputs:{text:'A'},parameters:{phrase:'ALLTHEPEOPLEAREDEADB'}});
const bellaso: string = cube.execute({ operation: 'cipher.bellaso', version: 1, inputs: { text: 'A' }, parameters: { key: 'A', unmapped: 'reject' } }).text;
void bellaso;
// @ts-expect-error A canonical countersign is required.
cube.execute({ operation: 'cipher.bellaso', version: 1, inputs: { text: 'A' }, parameters: {} });
// @ts-expect-error Folding is not a Bellaso preservation policy.
cube.execute({ operation: 'cipher.bellaso', version: 1, inputs: { text: 'A' }, parameters: { key: 'A', unmapped: 'fold' } });
const bytes: Uint8Array = cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: 'AP8=' } }).output;
const text: string = cube.execute({ operation: 'encoding.base64.encode', version: 1, inputs: { input: bytes } }).output;
const joined: string = cube.execute({ operation: 'text.concat', version: 1, inputs: { left: 'Cube', right: 'Arc' }, parameters: { separator: '' } }).output;
const sum: number = cube.execute({ operation: 'number.add', version: 1, inputs: { left: 1, right: 2 } }).output;
void joined;
void sum;
cube.runRecipe({
  format_version: 1, engine_api: 1,
  input: { source: 'inline', value: { type: 'bytes', data: bytes } },
  output: { type: 'text' },
  steps: [{ operation: 'encoding.hex.encode', version: 1 }],
} satisfies RecipeDefinition);
void text;
// @ts-expect-error Text cannot be substituted for exact binary input.
cube.execute({ operation: 'encoding.base64.encode', version: 1, inputs: { input: 'text' } });
// @ts-expect-error Unknown ports are not silently dropped.
cube.execute({ operation: 'encoding.base64.decode', version: 1, inputs: { input: '', extra: '' } });
const image = cube.execute({ operation: 'image.decode', version: 1, inputs: { input: bytes } });
void image;
const artifact = cube.execute({ operation: 'file.info.inspect', version: 2, inputs: { input: bytes } });
void artifact;
// @ts-expect-error Unknown identities are not executable operations.
cube.execute({ operation: 'not.a.cube.operation', version: 1, inputs: { input: bytes } });
// @ts-expect-error Bytes require an explicitly typed buffer.
cube.runRecipe('', { message: { type: 'bytes', data: [0, 255] } });
// @ts-expect-error Named multi-input ports are required together.
cube.execute({ operation: 'text.concat', version: 1, inputs: { left: 'Cube' } });
// @ts-expect-error Real parameter schemas reject unknown names.
cube.execute({ operation: 'text.concat', version: 1, inputs: { left: '', right: '' }, parameters: { shell: true } });
// @ts-expect-error The Brainfuck operation requires its actual contract version.
cube.execute({ operation: 'esolang.brainfuck.execute', version: 1, inputs: { source: ',.', input: bytes } });
const rot8000: string = cube.execute({ operation: 'cipher.rot8000', version: 1, inputs: { text: 'A 😀' } }).text;
const paired: string = cube.execute({ operation: 'cipher.vatsyayana', version: 1, inputs: { text: 'Q' }, parameters: { pairs: [['Q','W']] } }).text;
const historical = cube.execute({ operation: 'cipher.great_cipher.decode', version: 1, inputs: { tokens: ['55','502'] } }).report;
const alternatives: readonly string[] = historical.units[0].alternatives;
void rot8000; void paired; void alternatives;
// @ts-expect-error A codebook uses explicitly segmented units, not joined plaintext.
cube.execute({ operation: 'cipher.great_cipher.encode', version: 1, inputs: { text: 'LesEnnemis' } });
// @ts-expect-error Vatsyayana requires an explicit pairing key.
cube.execute({ operation: 'cipher.vatsyayana', version: 1, inputs: { text: 'A' }, parameters: {} });
