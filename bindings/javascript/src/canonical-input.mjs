import { CubeError } from './errors.mjs';
import { integerDecimal } from './transport.mjs';

// Transport only: canonical schemas decide which fields accept exact integers.
export function canonicalIntegers(value, schema, bounds, code = 'invalid_input') {
  let nodes = 0;
  function choices(schema, root, value, seen = new Set()) {
    if (!schema || typeof schema !== 'object') return [];
    if (schema.$ref) {
      if (seen.has(schema.$ref)) return [];
      const target = schema.$ref.split('/').slice(1).reduce((v, key) => v?.[key.replaceAll('~1', '/').replaceAll('~0', '~')], root);
      return choices(target, root, value, new Set([...seen, schema.$ref]));
    }
    if (schema['x-cube-integer']) return [schema['x-cube-integer']];
    if (schema.anyOf || schema.oneOf) return (schema.anyOf ?? schema.oneOf).flatMap(s => choices(s, root, value, seen));
    if (schema.allOf) return schema.allOf.flatMap(s => choices(s, root, value, seen));
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      if (Object.entries(schema.properties ?? {}).some(([key, s]) => s.const !== undefined && value[key] !== s.const)) return [];
    }
    return [schema];
  }
  function visit(value, schemas, depth) {
    if (++nodes > bounds.documentNodes || depth > bounds.documentDepth) throw new CubeError('resource_limit', 'Invocation exceeds its transport structure budget.');
    const candidates = schemas.flatMap(s => choices(s, schema, value));
    if (typeof value === 'bigint') {
      const decimal = integerDecimal(value);
      const valid = candidates.some(s => {
        if (s.type !== 'integer') return false;
        const minimum = s['x-cube-minimum-decimal'] ?? s.minimum;
        const maximum = s['x-cube-maximum-decimal'] ?? s.maximum;
        return (minimum === undefined || value >= BigInt(minimum)) &&
          (maximum === undefined || value <= BigInt(maximum)) &&
          (!s.enum || s.enum.some(item => BigInt(item) === value));
      });
      if (!valid) throw new CubeError(code, 'An integer does not match its declared field type or bounds.');
      return value >= -9007199254740991n && value <= 9007199254740991n ? Number(value) : decimal;
    }
    if (value instanceof Uint8Array || value === null) return value;
    if (Array.isArray(value)) return value.map((item, index) => visit(item, candidates.map(s => s.prefixItems?.[index] ?? s.items), depth + 1));
    if (value && typeof value === 'object') {
      if (Object.getPrototypeOf(value) !== Object.prototype) throw new CubeError(code, 'Use plain named JSON values.');
      return Object.fromEntries(Object.entries(value).map(([key, item]) => [key,
        visit(item, candidates.map(s => s.properties?.[key] ?? s.additionalProperties), depth + 1)]));
    }
    return value;
  }
  return visit(value, [schema], 0);
}
