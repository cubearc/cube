import { CubeError, requireText } from './errors.mjs';

export function immutable(value) {
  if (value && typeof value === 'object') { for (const item of Object.values(value)) immutable(item); Object.freeze(value); }
  return value;
}

// JSON transport only. Core owns all operations, byte codecs and validation.
export function integerDecimal(value) {
  // Compare magnitudes before decimal conversion, even for caller-created huge BigInts.
  if (value >= 10n ** 40n || value <= -(10n ** 39n)) throw new CubeError('invalid_input', 'Integer exceeds canonical transport bounds.');
  return value.toString();
}

export function stringifyExact(input, encodeBytes, { maxBytes = 256 * 1024, maxDepth = 16, maxNodes = 4096 } = {}) {
  let nodes = 0;
  let used = 0;
  const limit = () => new CubeError('resource_limit', 'JSON document exceeds its transport byte budget.');
  const consume = value => {
    if (value.length > maxBytes - used) throw limit();
    used += new TextEncoder().encode(value).byteLength;
    if (used > maxBytes) throw limit();
    return value;
  };
  const quoted = value => {
    if (value.length > maxBytes - used) throw limit();
    return consume(JSON.stringify(requireText(value)));
  };
  const encoded = value => {
    if (value.byteLength > Math.min(maxBytes - used, 1024 * 1024)) throw limit();
    return encodeBytes(value);
  };
  const visit = (value, depth) => {
    if (++nodes > maxNodes || depth > maxDepth) throw new CubeError('resource_limit', 'JSON exceeds the node or nesting budget.');
    if (value === null) return consume('null');
    if (typeof value === 'string') return quoted(value);
    if (typeof value === 'boolean') return consume(JSON.stringify(value));
    if (typeof value === 'bigint') return consume(integerDecimal(value));
    if (typeof value === 'number') {
      if (!Number.isFinite(value) || (Number.isInteger(value) && !Number.isSafeInteger(value))) {
        throw new CubeError('invalid_input', 'Use bigint or an exact JSON document for integers outside the safe Number range.');
      }
      return consume(JSON.stringify(value));
    }
    if (value instanceof Uint8Array) return visit({ base64: encoded(value) }, depth + 1);
    if (Array.isArray(value)) {
      const parts = [consume('[')];
      for (let index = 0; index < value.length; index++) {
        if (index) parts.push(consume(','));
        parts.push(visit(value[index], depth + 1));
      }
      parts.push(consume(']')); return parts.join('');
    }
    if (typeof value === 'object' && Object.getPrototypeOf(value) === Object.prototype) {
      if (value.type === 'bytes' && value.data instanceof Uint8Array) {
        if (Object.keys(value).some(key => !['type', 'data'].includes(key))) throw new CubeError('invalid_input', 'Bytes values require exactly type and data.');
        return visit({ type: 'bytes', base64: encoded(value.data) }, depth + 1);
      }
      const parts = [consume('{')];
      for (const key of Object.keys(value)) {
        if (parts.length > 1) parts.push(consume(','));
        parts.push(quoted(key), consume(':'), visit(value[key], depth + 1));
      }
      parts.push(consume('}')); return parts.join('');
    }
    throw new CubeError('invalid_input', 'Use plain JSON values and explicit Uint8Array data.');
  };
  return visit(input, 0);
}

export function boundedDocument(value, maxBytes = 256 * 1024) {
  requireText(value);
  if (value.length > maxBytes || new TextEncoder().encode(value).byteLength > maxBytes) {
    throw new CubeError('resource_limit', 'JSON document exceeds its transport byte budget.');
  }
  return value;
}

export function captureIndices(values) {
  if (!Array.isArray(values) || values.length > 64 || values.some(value => !Number.isInteger(value) || value < 0 || value >= 64)) {
    throw new CubeError('invalid_parameter', 'Captures require at most 64 step indices in 0..63.');
  }
  return Uint32Array.from(values);
}

export function executionFromWire(value, decodeBytes) {
  const tagged = item => item.type === 'bytes'
    ? { type: 'bytes', data: decodeBytes(item.base64) }
    : item;
  return {
    ...value,
    output: tagged(value.output),
    intermediates: Object.fromEntries(Object.entries(value.intermediates).map(([key, item]) => [key, tagged(item)])),
  };
}
