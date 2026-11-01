import { createCube, type RecipeDefinition } from '../../dist/node.mjs';

const cube = await createCube();
const primeTokens: readonly string[]=cube.execute({operation:'cipher.prime_number.encode',version:1,inputs:{text:'AB'},parameters:{profile:'indexed_primes_v1',index:{alphabet:'AB',first_index:1,max_prime:3}}}).tokens;
const multiset=cube.execute({operation:'cipher.prime_multiplication.decode',version:1,inputs:{products:['6']},parameters:{profile:'prime_multiset_v1',index:{alphabet:'AB',first_index:1,max_prime:3},max_factor_checks:10}}).groups[0];
const exactProduct: string=multiset.product;
const multiplicity: number=multiset.counts[0].count;
const residues: readonly string[]=cube.execute({operation:'cipher.modulo.encode',version:1,inputs:{text:'A'},parameters:{profile:'affine_residue_v1',alphabet:'AB',modulus:2,multiplier:'1',offset:'0',token_origin:1}}).tokens;
const trits: readonly string[]=cube.execute({operation:'cipher.triliteral.encode',version:1,inputs:{text:'AB'},parameters:{profile:'ordered_ternary_v1',alphabet:'AB',trits:'012'}}).tokens;
void primeTokens; void exactProduct; void multiplicity; void residues; void trits;
const solresol:readonly string[]=cube.execute({operation:'symbol.solresol.encode',version:1,inputs:{terms:['non','oui']},parameters:{profile:'gajewski_1902_monosyllables_v1'}}).tokens;
const glosses:readonly string[]=cube.execute({operation:'symbol.solresol.decode',version:1,inputs:{tokens:['do']},parameters:{profile:'gajewski_1902_monosyllables_v1'}}).readings[0].alternatives;
void solresol;void glosses;
// @ts-expect-error A source-qualified finite lexicon must be selected explicitly.
cube.execute({operation:'symbol.solresol.encode',version:1,inputs:{terms:[]},parameters:{}});
const phrase: string=cube.execute({operation:'cipher.key_phrase.encode',version:1,inputs:{text:'BHQ'},parameters:{phrase:'GIVEMELIBERTYORGIVEMEDEATH'}}).text;
const phraseReadings=cube.execute({operation:'cipher.key_phrase.decode',version:1,inputs:{text:'I'},parameters:{phrase:'GIVEMELIBERTYORGIVEMEDEATH'}});
const possibleText:string|null=phraseReadings.text;
const wordReport=cube.execute({operation:'cipher.arnold.encode',version:1,inputs:{pages:['A'],words:['A']},parameters:{profile:'arnold_paged_words_v1',source_id:'synthetic',index_base:1,choice:'first'}});
const exactWords:readonly string[]=wordReport.words;
void phrase;void possibleText;void exactWords;
// @ts-expect-error A phrase inverse can remain ambiguous.
const guessedPlaintext:string=phraseReadings.text;
void guessedPlaintext;
// @ts-expect-error Paginated whole-word codes require explicit pages.
cube.execute({operation:'cipher.arnold.encode',version:1,inputs:{words:['A']},parameters:{profile:'arnold_paged_words_v1',source_id:'synthetic',index_base:1,choice:'first'}});
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

const alphabetTokens:readonly string[]=cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:['A','CH']},parameters:{profile:'aurebesh_ucsur_2016_v1'}}).tokens;
const alphabetReadings:readonly string[]=cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:['\ueb40']},parameters:{profile:'standard_galactic_ucsur_2019_v1'}}).readings[0].alternatives;
void alphabetTokens;void alphabetReadings;
// @ts-expect-error Named source profile is required.
cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:[]},parameters:{}});
// @ts-expect-error Unqualified display names are not profile IDs.
cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:[]},parameters:{profile:'Aurebesh'}});
