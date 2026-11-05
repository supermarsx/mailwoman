// A pool of sanitiser workers, separate from the crypto worker.
//
// - lazily spawned up to `size` (default clamp(hardwareConcurrency - 1, 1, 4));
// - the wasm is compiled ONCE (`loadModule`) and the compiled module is posted to each worker;
// - one job per worker at a time; idle worker first, then spawn, then queue;
// - bounded queue; `high` (the open message) runs before `low` (prefetch); a full queue
//   drops the oldest low job to admit a high one and refuses a low one;
// - an AbortSignal drops a queued job, or makes an in-flight job's result ignored;
// - a crashed worker is replaced and its job retried once on another worker; after that,
//   or when no worker can be made, `fallback` (the server's /api/sanitize) answers.

import type { FromWorker, SanitizeRequest, ToWorker } from './protocol.ts';

export type Priority = 'high' | 'low';

export interface WorkerLike {
  postMessage(message: ToWorker): void;
  terminate(): void;
  onmessage: ((ev: MessageEvent<FromWorker>) => void) | null;
  onerror: ((ev: ErrorEvent) => void) | null;
  onmessageerror?: ((ev: MessageEvent) => void) | null;
}

export interface PoolOptions {
  size: number;
  createWorker: () => WorkerLike;
  /** Compile the wasm once; shared by every worker. */
  loadModule: () => Promise<WebAssembly.Module>;
  /** Answer a job the workers could not (crash twice, no worker support, queue full). */
  fallback?: (req: SanitizeRequest) => Promise<string>;
  maxQueue?: number;
}

export class QueueFullError extends Error {
  constructor() { super('sanitizer queue full'); this.name = 'QueueFullError'; }
}

interface Job {
  id: number;
  req: SanitizeRequest;
  priority: Priority;
  attempts: number;
  signal?: AbortSignal;
  onAbort?: () => void;
  resolve(html: string): void;
  reject(error: unknown): void;
  settled: boolean;
}

interface Slot {
  worker: WorkerLike;
  ready: Promise<void>;
  job: Job | null;
  dead: boolean;
}

export interface PoolStats { workers: number; busy: number; queued: number; spawned: number; crashes: number }

function abortError(): Error {
  const e = new Error('sanitize aborted');
  e.name = 'AbortError';
  return e;
}

export class SanitizerPool {
  private readonly slots: Slot[] = [];
  private readonly queue: Job[] = [];
  private module: Promise<WebAssembly.Module> | null = null;
  private seq = 0;
  private spawned = 0;
  private crashes = 0;
  private disposed = false;

  constructor(private readonly opts: PoolOptions) {}

  run(req: SanitizeRequest, options: { priority?: Priority; signal?: AbortSignal } = {}): Promise<string> {
    const signal = options.signal;
    if (this.disposed) return this.fallback(req);
    if (signal?.aborted) return Promise.reject(abortError());
    const priority = options.priority ?? 'high';
    return new Promise<string>((resolve, reject) => {
      const job: Job = {
        id: (this.seq += 1), req, priority, attempts: 0, resolve, reject, settled: false,
        ...(signal !== undefined ? { signal } : {}),
      };
      if (signal !== undefined) {
        job.onAbort = () => this.abort(job);
        signal.addEventListener('abort', job.onAbort, { once: true });
      }
      if (!this.admit(job)) return;
      this.pump();
    });
  }

  stats(): PoolStats {
    return {
      workers: this.slots.length,
      busy: this.slots.filter((s) => s.job !== null).length,
      queued: this.queue.length,
      spawned: this.spawned,
      crashes: this.crashes,
    };
  }

  /** Terminate every worker and refuse queued work (teardown). */
  dispose(): void {
    this.disposed = true;
    for (const slot of this.slots.splice(0)) {
      slot.dead = true;
      slot.worker.terminate();
      const job = slot.job;
      slot.job = null;
      if (job) this.settle(job, () => job.reject(abortError()));
    }
    for (const job of this.queue.splice(0)) this.settle(job, () => job.reject(abortError()));
  }

  // ── queue ────────────────────────────────────────────────────────────────────

  private admit(job: Job): boolean {
    const max = this.opts.maxQueue ?? 64;
    if (this.queue.length >= max) {
      if (job.priority === 'low') {
        this.settle(job, () => job.reject(new QueueFullError()));
        return false;
      }
      const lowAt = this.queue.findIndex((j) => j.priority === 'low');
      if (lowAt === -1) {
        this.settle(job, () => void this.fallback(job.req).then(job.resolve, job.reject));
        return false;
      }
      const [dropped] = this.queue.splice(lowAt, 1);
      this.settle(dropped!, () => dropped!.reject(new QueueFullError()));
    }
    if (job.priority === 'high') {
      const firstLow = this.queue.findIndex((j) => j.priority === 'low');
      if (firstLow === -1) this.queue.push(job);
      else this.queue.splice(firstLow, 0, job);
    } else {
      this.queue.push(job);
    }
    return true;
  }

  private abort(job: Job): void {
    if (job.settled) return;
    const at = this.queue.indexOf(job);
    if (at !== -1) this.queue.splice(at, 1);
    // In flight: the worker finishes and its reply is ignored (`settled`).
    this.settle(job, () => job.reject(abortError()));
  }

  private settle(job: Job, act: () => void): void {
    if (job.settled) return;
    job.settled = true;
    if (job.signal && job.onAbort) job.signal.removeEventListener('abort', job.onAbort);
    act();
  }

  // ── workers ──────────────────────────────────────────────────────────────────

  private pump(): void {
    while (this.queue.length > 0) {
      let slot = this.slots.find((s) => s.job === null && !s.dead);
      if (slot === undefined && this.slots.length < Math.max(1, this.opts.size)) slot = this.spawn();
      if (slot === undefined) {
        // Nothing can run these jobs and nothing is working that would free a slot.
        if (this.slots.length === 0) this.failQueue();
        return;
      }
      const job = this.queue.shift()!;
      if (job.settled) continue;
      this.dispatch(slot, job);
    }
  }

  private failQueue(): void {
    for (const queued of this.queue.splice(0)) {
      this.settle(queued, () => void this.fallback(queued.req).then(queued.resolve, queued.reject));
    }
  }

  private spawn(): Slot | undefined {
    let worker: WorkerLike;
    try {
      worker = this.opts.createWorker();
    } catch {
      return undefined; // no worker support (or blocked): callers fall back
    }
    this.spawned += 1;
    this.module ??= this.opts.loadModule();
    const slot: Slot = {
      worker, job: null, dead: false,
      ready: this.module.then((module) => { worker.postMessage({ type: 'init', module }); }),
    };
    worker.onmessage = (ev) => this.onReply(slot, ev.data);
    worker.onerror = () => this.onCrash(slot);
    worker.onmessageerror = () => this.onCrash(slot);
    this.slots.push(slot);
    return slot;
  }

  private dispatch(slot: Slot, job: Job): void {
    slot.job = job;
    job.attempts += 1;
    slot.ready.then(
      () => { if (!slot.dead && slot.job === job) slot.worker.postMessage({ type: 'job', id: job.id, req: job.req }); },
      () => this.moduleFailed(slot, job),
    );
  }

  private onReply(slot: Slot, msg: FromWorker): void {
    const job = slot.job;
    if (job === null || msg.id !== job.id) return;
    slot.job = null;
    this.settle(job, () => (msg.ok ? job.resolve(msg.html) : job.reject(new Error(msg.error))));
    this.pump();
  }

  private onCrash(slot: Slot): void {
    if (slot.dead) return;
    this.crashes += 1;
    slot.dead = true;
    slot.worker.terminate();
    const at = this.slots.indexOf(slot);
    if (at !== -1) this.slots.splice(at, 1);
    const job = slot.job;
    slot.job = null;
    if (job && !job.settled) {
      if (job.attempts < 2 && !this.disposed) {
        this.queue.unshift(job); // retry once, on a fresh worker
      } else {
        this.settle(job, () => void this.fallback(job.req).then(job.resolve, job.reject));
      }
    }
    this.pump();
  }

  private moduleFailed(slot: Slot, job: Job): void {
    // The wasm could not be compiled: workers are useless, so answer from the fallback.
    this.module = null;
    slot.dead = true;
    slot.worker.terminate();
    const at = this.slots.indexOf(slot);
    if (at !== -1) this.slots.splice(at, 1);
    slot.job = null;
    this.settle(job, () => void this.fallback(job.req).then(job.resolve, job.reject));
    this.failQueue();
  }

  private fallback(req: SanitizeRequest): Promise<string> {
    return this.opts.fallback ? this.opts.fallback(req) : Promise.reject(new Error('sanitizer unavailable'));
  }
}

/** clamp(hardwareConcurrency - 1, 1, 4); `override` (>0) replaces the auto size. */
export function poolSize(hardwareConcurrency: number | undefined, override = 0): number {
  if (override > 0) return Math.min(8, Math.floor(override));
  return Math.min(4, Math.max(1, (hardwareConcurrency ?? 2) - 1));
}
