import type { BuildIdentity, Cube, Execution, JsonValue, PlanInfo, RecipeDefinition, Value } from './index.js';
import type { DirectInputs, DirectOutputs, DirectVersions, DirectConfiguration, OtherDirectOperationId } from './direct-operations.js';
export { CubeError } from './index.js';

export interface WorkerOptions {
  readonly signal?: AbortSignal;
  readonly context?: JsonValue;
  readonly timeoutMs?: number;
}
export interface WorkerResult<T> {
  readonly requestId: number;
  readonly generation: number;
  readonly context: JsonValue;
  readonly value: T;
}
export interface WorkerCube {
  readonly build: BuildIdentity;
  readonly pipelineAdapters: Cube['pipelineAdapters'];
  readonly executors: Cube['executors'];
  readonly limits: Cube['limits'];
  execute(request: { readonly operation: 'encoding.base64.encode'; readonly version: 1; readonly inputs: { readonly input: Uint8Array }; readonly parameters?: Record<string, never> }, options?: WorkerOptions): Promise<WorkerResult<{ output: string }>>;
  execute(request: { readonly operation: 'encoding.base64.decode'; readonly version: 1; readonly inputs: { readonly input: string }; readonly parameters?: Record<string, never> }, options?: WorkerOptions): Promise<WorkerResult<{ output: Uint8Array }>>;
  execute<Id extends OtherDirectOperationId>(request: { readonly operation: Id; readonly version: DirectVersions[Id]; readonly inputs: DirectInputs[Id] } & DirectConfiguration[Id], options?: WorkerOptions): Promise<WorkerResult<DirectOutputs[Id]>>;
  catalogJson(options?: WorkerOptions): Promise<WorkerResult<string>>;
  recipeJson(definition: RecipeDefinition | string, options?: WorkerOptions): Promise<WorkerResult<string>>;
  validateRecipe(definition: RecipeDefinition | string, options?: WorkerOptions): Promise<WorkerResult<PlanInfo>>;
  runRecipe(definition: RecipeDefinition | string, bindings?: Readonly<Record<string, Value>> | string, execution?: { readonly captures?: readonly number[] }, options?: WorkerOptions): Promise<WorkerResult<Execution>>;
  /** Terminates this generation; subsequent calls initialize a new worker. */
  terminate(): void;
}
export function createWorkerCube(options?: { readonly workerUrl?: URL; readonly wasmUrl?: URL; readonly initializationTimeoutMs?: number }): Promise<WorkerCube>;
