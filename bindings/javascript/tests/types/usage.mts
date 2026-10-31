import { createCube, type RecipeDefinition } from '../../dist/node.mjs';

const cube = await createCube();
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
// @ts-expect-error The complete catalog is not the direct invocation union.
cube.execute({ operation: 'image.decode', version: 1, inputs: { input: bytes } });
// @ts-expect-error Bytes require an explicitly typed buffer.
cube.runRecipe('', { message: { type: 'bytes', data: [0, 255] } });
// @ts-expect-error Named multi-input ports are required together.
cube.execute({ operation: 'text.concat', version: 1, inputs: { left: 'Cube' } });
// @ts-expect-error Real parameter schemas reject unknown names.
cube.execute({ operation: 'text.concat', version: 1, inputs: { left: '', right: '' }, parameters: { shell: true } });
// @ts-expect-error The Brainfuck operation requires its actual contract version.
cube.execute({ operation: 'esolang.brainfuck.execute', version: 1, inputs: { source: ',.', input: bytes } });
