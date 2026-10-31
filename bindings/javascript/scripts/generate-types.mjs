import { writeFile } from 'node:fs/promises';

function schemaType(schema, output = false, root = schema, seen = new Set()) {
  if (!schema || schema === true) return 'JsonValue';
  if (schema === false) return 'never';
  if (schema.$ref) {
    if (seen.has(schema.$ref)) return 'JsonValue';
    const target = schema.$ref.split('/').slice(1).reduce((value, key) => value?.[key.replaceAll('~1', '/').replaceAll('~0', '~')], root);
    return schemaType(target, output, root, new Set([...seen, schema.$ref]));
  }
  if (schema['x-cube-integer']) return output ? '(number | string)' : '(number | bigint | string)';
  if (schema.const !== undefined) return JSON.stringify(schema.const);
  if (schema.enum) return schema.enum.map(value => JSON.stringify(value)).join(' | ');
  if (schema.anyOf || schema.oneOf) return `(${(schema.anyOf ?? schema.oneOf).map(item => schemaType(item, output, root, seen)).join(' | ')})`;
  if (schema.allOf) return `(${schema.allOf.map(item => schemaType(item, output, root, seen)).join(' & ')})`;
  if (Array.isArray(schema.type)) return `(${schema.type.map(type => schemaType({ ...schema, type }, output, root, seen)).join(' | ')})`;
  switch (schema.type) {
    case 'null': return 'null';
    case 'string': return 'string';
    case 'boolean': return 'boolean';
    case 'number': return 'number';
    case 'integer': return output ? 'number' : '(number | bigint)';
    case 'array':
      return schema.prefixItems
        ? `readonly [${schema.prefixItems.map(item => schemaType(item, output, root, seen)).join(', ')}]`
        : `readonly (${schemaType(schema.items, output, root, seen)})[]`;
    case 'object': {
      const properties = Object.entries(schema.properties ?? {}).map(([name, value]) => `readonly ${JSON.stringify(name)}${schema.required?.includes(name) ? '' : '?'}: ${schemaType(value, output, root, seen)}`);
      if (schema.additionalProperties !== false) properties.push('readonly [key: string]: JsonValue');
      return properties.length ? `{ ${properties.join('; ')} }` : 'Record<string, never>';
    }
    default: return 'JsonValue';
  }
}

function portType(representation, port, output) {
  if (representation === 'text') return 'string';
  if (representation === 'bytes') return 'Uint8Array';
  if (representation === 'byte_array') return output ? 'Uint8Array' : '(Uint8Array | readonly number[])';
  return schemaType(port.schema, output);
}

export async function generateDirectTypes(path, executors, catalog) {
  const definitions = executors.map(executor => ({ ...executor, contract: catalog.operations.find(operation => operation.operation === executor.operation) }));
  const ports = (entry, output) => {
    const contracts = output ? entry.contract.outputs : entry.contract.inputs;
    const representations = output ? entry.outputs : entry.inputs;
    return `{ ${contracts.map(port => `readonly ${JSON.stringify(port.name)}${port.required ? '' : '?'}: ${portType(representations[port.name], port, output)}`).join('; ')} }`;
  };
  const table = (name, type) => `export interface ${name} {\n${definitions.map(entry => `  ${JSON.stringify(entry.operation)}: ${type(entry)};`).join('\n')}\n}\n`;
  const text = "// Generated from actual Wasm executors and canonical schemas; do not edit.\nimport type { JsonValue } from './index.js';\n" +
    table('DirectInputs', entry => ports(entry, false)) + table('DirectOutputs', entry => ports(entry, true)) +
    table('DirectParameters', entry => schemaType(entry.contract.parameters)) + table('DirectVersions', entry => String(entry.version)) +
    table('DirectConfiguration', entry => `{ readonly parameters${entry.contract.parameters.required?.length ? '' : '?'}: DirectParameters[${JSON.stringify(entry.operation)}] }`) +
    'export type DirectOperationId = keyof DirectInputs;\n' +
    "export type OtherDirectOperationId = Exclude<DirectOperationId, 'encoding.base64.encode' | 'encoding.base64.decode'>;\n";
  await writeFile(path, text);
}
