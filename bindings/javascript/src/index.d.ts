import type { PipelineOperationId } from './operation-ids.js';
import type { DirectInputs, DirectOutputs, DirectVersions, DirectConfiguration, OtherDirectOperationId } from './direct-operations.js';
export type { DirectOperationId, DirectInputs, DirectOutputs, DirectParameters } from './direct-operations.js';
export type { PipelineOperationId } from './operation-ids.js';

export class CubeError extends Error {
  readonly code: string;
  readonly operation?: string;
  readonly parameter?: string;
  readonly step?: number;
  readonly offset?: number | string;
  readonly reason?: string;
}

export type Kind = 'text' | 'bytes' | 'byte_range' | 'forensic_report';
export type JsonValue = null | boolean | string | number | bigint | Uint8Array | readonly JsonValue[] | { readonly [key: string]: JsonValue };
export type Value =
  | { readonly type: 'text'; readonly data: string }
  | { readonly type: 'bytes'; readonly data: Uint8Array }
  | { readonly type: 'byte_range'; readonly offset: number | bigint; readonly length: number | bigint | null }
  | { readonly type: 'forensic_report'; readonly data: { readonly [key: string]: JsonValue } };
export type ResultValue = Exclude<Value, { type: 'byte_range' }> |
  { readonly type: 'byte_range'; readonly offset: number | string; readonly length: number | string | null };
export interface Step {
  readonly operation: PipelineOperationId;
  readonly version: number;
  readonly parameters?: Readonly<Record<string, JsonValue>>;
}
export interface Limits {
  readonly max_steps?: number;
  readonly max_value_bytes?: number;
  readonly max_work_bytes?: number;
  readonly max_capture_bytes?: number;
}
export interface RecipeDefinition {
  readonly format_version: 1;
  readonly engine_api: 1;
  readonly input: { readonly source: 'slot'; readonly name: string; readonly type: Kind } |
    { readonly source: 'inline'; readonly value: Value };
  readonly output: { readonly type: Kind };
  readonly steps: readonly Step[];
  readonly limits?: Limits;
}
export interface PlanInfo {
  readonly format_version: number;
  readonly input_slot: string | null;
  readonly plan: {
    readonly engine_api: number;
    readonly engine_version: string;
    readonly input_type: Kind;
    readonly output_type: Kind;
    readonly steps: readonly { step: number; operation: PipelineOperationId; version: number; input_type: Kind; output_type: Kind }[];
    readonly limits: Required<Limits>;
  };
}
export interface Execution {
  readonly output: ResultValue;
  readonly intermediates: Readonly<Record<string, ResultValue>>;
  readonly masked_audits?: Readonly<Record<string, JsonValue>>;
}
export interface BuildIdentity {
  readonly formatVersion: number;
  readonly sdkVersion: string;
  readonly coreRevision: string;
  readonly sourceDirty: boolean;
  readonly sourceSha256: string;
  readonly catalogSha256: string;
  readonly wasmSha256: string;
}
export interface Cube {
  readonly build: BuildIdentity;
  readonly limits: { readonly linearMemoryBytes: number; readonly direct: Readonly<Record<string, number>>; readonly recipe: Readonly<Record<string, number>> };
  readonly pipelineAdapters: readonly { readonly operation: PipelineOperationId; readonly version: number; readonly input_type: Kind; readonly output_type: Kind; readonly parameters: readonly { readonly name: string; readonly description: string; readonly required: boolean; readonly default: JsonValue; readonly constraints: { readonly type: string; readonly [key: string]: JsonValue } }[] }[];
  readonly executors: readonly { readonly operation: string; readonly version: number; readonly effects: 'pure'; readonly inputs: Readonly<Record<string, string>>; readonly outputs: Readonly<Record<string, string>>; readonly parameters: JsonValue }[];
  execute(request: { readonly operation: 'encoding.base64.encode'; readonly version: 1; readonly inputs: { readonly input: Uint8Array }; readonly parameters?: Record<string, never> }): { output: string };
  execute(request: { readonly operation: 'encoding.base64.decode'; readonly version: 1; readonly inputs: { readonly input: string }; readonly parameters?: Record<string, never> }): { output: Uint8Array };
  execute<Id extends OtherDirectOperationId>(request: { readonly operation: Id; readonly version: DirectVersions[Id]; readonly inputs: DirectInputs[Id] } & DirectConfiguration[Id]): DirectOutputs[Id];
  /** Complete canonical wire; use decimal annotations for wide schema bounds. */
  catalogJson(): string;
  /** Exact bounded transport serialization; call validateRecipe separately. */
  recipeJson(definition: RecipeDefinition | string): string;
  validateRecipe(definition: RecipeDefinition | string): PlanInfo;
  runRecipe(definition: RecipeDefinition | string, bindings?: Readonly<Record<string, Value>> | string, options?: { readonly captures?: readonly number[] }): Execution;
}
export function createCube(options?: { readonly wasm?: URL | Uint8Array }): Promise<Cube>;
