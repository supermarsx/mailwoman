// SharedWorker store protocol helpers (plan §2.6, contracts/worker.ts).
//
// The tab↔worker (and BroadcastChannel-fallback) wire is a `WorkerEnvelope`
// `{id, kind, method, params, result, error}`. This module builds envelopes and
// correlates `res` frames back to their `req` promises; `workerCore.ts` (worker
// side) and `proxy.ts` (tab side) share it.

import type { StateBroadcast, WorkerEnvelope } from '../contracts/worker.ts';

/** Structural view of a `MessagePort` / SharedWorker port / BroadcastChannel. */
export interface PortLike {
  postMessage(data: unknown): void;
  onmessage: ((ev: { data: unknown }) => void) | null;
  start?(): void;
  close?(): void;
}

let seq = 0;
export function nextId(): string {
  seq += 1;
  return `${seq}-${Math.random().toString(36).slice(2, 8)}`;
}

export function reqEnvelope(method: string, params?: unknown): WorkerEnvelope {
  return { id: nextId(), kind: 'req', method, params };
}

export function resEnvelope(id: string, result?: unknown, error?: unknown): WorkerEnvelope {
  return { id, kind: 'res', result, error };
}

export function broadcastEnvelope(params: unknown): StateBroadcast {
  return { id: '', kind: 'broadcast', method: 'state', params };
}

/** Turn a thrown value into a structured-clone-safe payload for the `res` frame. */
export function serializeError(err: unknown): { message: string } {
  return { message: err instanceof Error ? err.message : String(err) };
}

interface Pending {
  resolve: (v: unknown) => void;
  reject: (e: unknown) => void;
  timer: ReturnType<typeof setTimeout> | undefined;
}

/** Correlates outbound `req` frames to their `res` frames (tab side). */
export class Correlator {
  private pending = new Map<string, Pending>();

  request(env: WorkerEnvelope, send: (e: WorkerEnvelope) => void, timeoutMs = 15_000): Promise<unknown> {
    return new Promise<unknown>((resolve, reject) => {
      let timer: ReturnType<typeof setTimeout> | undefined;
      if (timeoutMs > 0) {
        timer = setTimeout(() => {
          this.pending.delete(env.id);
          reject(new Error(`worker request "${env.method ?? '?'}" timed out`));
        }, timeoutMs);
      }
      this.pending.set(env.id, { resolve, reject, timer });
      send(env);
    });
  }

  /** Feed an inbound frame; resolves the matching pending req. Returns handled. */
  handle(env: WorkerEnvelope): boolean {
    if (env.kind !== 'res') return false;
    const p = this.pending.get(env.id);
    if (p === undefined) return false;
    this.pending.delete(env.id);
    if (p.timer !== undefined) clearTimeout(p.timer);
    if (env.error !== undefined && env.error !== null) p.reject(env.error);
    else p.resolve(env.result);
    return true;
  }

  rejectAll(reason: unknown): void {
    for (const p of this.pending.values()) {
      if (p.timer !== undefined) clearTimeout(p.timer);
      p.reject(reason);
    }
    this.pending.clear();
  }
}
