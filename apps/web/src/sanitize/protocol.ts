// Messages between the sanitiser pool and its workers.

/** Which `mw-sanitize` wasm export a job runs. */
export type SanitizeOp = 'strip' | 'reader' | 'readerSafe';

export interface SanitizeRequest {
  op: SanitizeOp;
  html: string;
  /** Reader ops only. */
  blockTrackers?: boolean;
  cleanLinks?: boolean;
}

export type ToWorker =
  | { type: 'init'; module: WebAssembly.Module }
  | { type: 'job'; id: number; req: SanitizeRequest };

export type FromWorker =
  | { type: 'result'; id: number; ok: true; html: string }
  | { type: 'result'; id: number; ok: false; error: string };
