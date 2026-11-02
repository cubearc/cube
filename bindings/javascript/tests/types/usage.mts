import { createCube, type RecipeDefinition } from '../../dist/node.mjs';

const cube = await createCube();
const m94Settings={profile:'m94_v1' as const,order:[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25] as const,row_offsets:[1]};
const m94Text:string=cube.execute({operation:'cipher.m94.encode',version:1,inputs:{text:'A',settings:m94Settings}}).text;
const wheelText:string=cube.execute({operation:'cipher.jefferson.decode',version:1,inputs:{text:'B',settings:{profile:'jefferson_configured_v1',wheels:['ABCDEFGHIJKLMNOPQRSTUVWXYZ'],order:[1],row_offsets:[1]}}}).text;
void m94Text;void wheelText;
// @ts-expect-error Historical rows must be explicitly selected for decoding.
cube.execute({operation:'cipher.m94.decode',version:1,inputs:{text:'B',settings:{profile:'m94_v1',order:m94Settings.order}}});
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

const geometric:readonly string[]=cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:['a']},parameters:{profile:'betamaze_cliff_2006_v1'}}).tokens;
void geometric;

const hylian:readonly string[]=cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:['hylian:r3c5']},parameters:{profile:'hylian_skyward_dcode_v1'}}).readings[0].alternatives;
void hylian;
for(const profile of ['al_bhed_english_ffx_v1','sheikah_botw_dcode_v1','daedric_names_dcode_v1','futurama_alien1_dcode_v1','bill_cipher_dcode_v1','gnommish_disney_2020_v1'] as const) {
  const tokens:readonly string[]=cube.execute({operation:'symbol.alphabet.encode',version:1,inputs:{units:['A']},parameters:{profile}}).tokens;
  void tokens;
}
// @ts-expect-error Other fictional editions are not silently inferred.
cube.execute({operation:'symbol.alphabet.decode',version:1,inputs:{tokens:[]},parameters:{profile:'hylian_twilight_princess'}});

const substitutionReport=cube.execute({operation:'analysis.substitution.solve',version:1,inputs:{request:{profile:'observed_bijection_v1',alphabet:'ABCDEFGHIJKLMNOPQRSTUVWXYZ',ciphertext:'ABBA',outside:'preserve'}}}).report;
const keyCount:string|undefined=substitutionReport.candidates[0]?.key_count;
void keyCount;
const familyReport=cube.execute({operation:'analysis.family.suggest',version:1,inputs:{request:{profile:'ascii_classical_evidence_v1',text:'ABC'}}}).report;
const evidenceRule:'nonuniform_coincidence_v1'|'periodic_columns_v1'|undefined=familyReport.suggestions[0]?.rule;
void evidenceRule;
// @ts-expect-error The substitution profile is explicit.
cube.execute({operation:'analysis.substitution.solve',version:1,inputs:{request:{alphabet:'ABCDEFGHIJKLMNOPQRSTUVWXYZ',ciphertext:'ABBA',outside:'preserve'}}});
// @ts-expect-error Family guesses use finite reviewed rules, not arbitrary labels.
cube.execute({operation:'analysis.family.suggest',version:1,inputs:{request:{profile:'ascii_classical_evidence_v1',text:'ABC',rules:['detect_aes']}}});

const wheatstoneReport=cube.execute({operation:'cipher.wheatstone.encode',version:1,inputs:{text:'THE+QUARTER+',settings:{profile:'wheatstone_1879_v1',cipher_alphabet:'FBKSYRDLTZAGMUNHOVCIPWEJQX',initial_state:{plain_position:0,cipher_position:0},trace:true}}});
const wheatstonePosition:number=wheatstoneReport.final_state.plain_position;
const kryhaPosition:number=cube.execute({operation:'cipher.kryha.decode',version:1,inputs:{text:'H',settings:{profile:'kryha_callimahos_1973_v1',plain_alphabet:'ABCDEFGHIJKLMNOPQRSTUVWXYZ',cipher_alphabet:'HYDRAULICBEFGJKMNOPQSTVWXZ',initial_state:{rotation:0,selector:1}}}}).final_state.selector;
const albertiIndex:string=cube.execute({operation:'cipher.alberti.encode',version:1,inputs:{text:'A4',settings:{profile:'alberti_di_palma_2009_v1',initial_index:'c'}}}).final_index;
void wheatstonePosition;void kryhaPosition;void albertiIndex;
// @ts-expect-error The source-specific Kryha profile requires an explicit selector.
cube.execute({operation:'cipher.kryha.encode',version:1,inputs:{text:'A',settings:{profile:'kryha_callimahos_1973_v1',plain_alphabet:'ABCDEFGHIJKLMNOPQRSTUVWXYZ',cipher_alphabet:'ABCDEFGHIJKLMNOPQRSTUVWXYZ',initial_state:{rotation:0}}}});

const lorenzSettings={profile:'lorenz_sz42b_v1' as const,chi:['0'.repeat(41),'0'.repeat(31),'0'.repeat(29),'0'.repeat(26),'0'.repeat(23)] as const,psi:['0'.repeat(43),'0'.repeat(47),'0'.repeat(51),'0'.repeat(53),'0'.repeat(59)] as const,mu61:'1'.repeat(61),mu37:'0'.repeat(37),plaintext_limitation:true,initial_state:{chi:[0,0,0,0,0] as const,psi:[0,0,0,0,0] as const,mu61:0,mu37:0,chi2_previous:false,psi1_previous:false,plain5_previous:[false,false] as const}};
const lorenz=cube.execute({operation:'cipher.lorenz.encode',version:1,inputs:{codes:[0,31],settings:lorenzSettings}});
const lorenzCodes:readonly number[]=lorenz.codes;
const previousPlainBit:boolean=lorenz.final_state.plain5_previous[0];
void lorenzCodes;void previousPlainBit;
// @ts-expect-error All twelve wheel settings and delayed history are required.
cube.execute({operation:'cipher.lorenz.encode',version:1,inputs:{codes:[],settings:{profile:'lorenz_sz40_v1'}}});

const hagelin=cube.execute({operation:'cipher.hagelin.encode',version:1,inputs:{text:'ABC',settings:{profile:'c36_fixed25_bouch_v1',pins:['0'.repeat(25),'0'.repeat(23),'0'.repeat(21),'0'.repeat(19),'0'.repeat(17)] as const,initial_state:[0,0,0,0,0] as const,slide:0}}});
const hagelinText:string=hagelin.text;
const hagelinState:readonly number[]=hagelin.final_state;
void hagelinText;void hagelinState;

const t52abSettings={profile:'siemens_t52ab_configured_v1' as const,wheel_set_id:'synthetic-types',cam_patterns:['1'.repeat(47),'1'.repeat(53),'1'.repeat(59),'1'.repeat(61),'1'.repeat(64),'1'.repeat(65),'1'.repeat(67),'1'.repeat(69),'1'.repeat(71),'1'.repeat(73)] as const,wheel_order:[0,1,2,3,4,5,6,7,8,9] as const,socket_pairs:[[1,3],[2,10],[4,5],[6,7],[8,9]] as const,initial_state:[0,0,0,0,0,0,0,0,0,0] as const};
const t52ab=cube.execute({operation:'cipher.t52ab.encode',version:1,inputs:{codes:[0,31],settings:t52abSettings}});
const t52abCodes:readonly number[]=t52ab.codes;
const t52abState:readonly number[]=t52ab.final_state;
void t52abCodes;void t52abState;

const nemaSettings={profile:'nema_model45_operational_v1' as const,rotors:['A','B','C','D'] as const,drive_wheels:[12,13,14,15] as const,initial_state:{windows:'MQQPZQSOEV'}};
const nema=cube.execute({operation:'cipher.nema.encode',version:1,inputs:{text:'A'.repeat(27),settings:nemaSettings}});
const nemaText:string=nema.text;
const nemaWindows:string=nema.final_state.windows;
void nemaText;void nemaWindows;
