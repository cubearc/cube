export class CubeError extends Error {
  constructor(code, message, context = {}) {
    super(message);
    this.name = 'CubeError';
    this.code = code;
    for (const field of ['operation', 'parameter', 'step', 'offset', 'reason']) {
      if (context[field] !== undefined && context[field] !== null) this[field] = context[field];
    }
  }
}

export function coreCall(action) {
  try { return action(); }
  catch (failure) {
    if (failure instanceof CubeError) throw failure;
    if (typeof failure === 'string') {
      try {
        const value = JSON.parse(failure);
        if (typeof value.code === 'string' && typeof value.message === 'string') {
          throw new CubeError(value.code, value.message, value);
        }
      } catch (error) {
        if (error instanceof CubeError) throw error;
      }
    }
    throw new CubeError('runtime_failure', 'The Wasm runtime failed; discard this invocation and restart its worker.');
  }
}

export function requireText(value) {
  if (typeof value !== 'string' || !value.isWellFormed()) {
    throw new CubeError('invalid_input', 'Expected text with valid Unicode scalar values.');
  }
  return value;
}
