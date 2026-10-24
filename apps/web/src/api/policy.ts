// The part of the admin security policy this client enforces itself (t29-e6).
//
// Server contract — `crates/mw-server/src/client_policy.rs`, session-authed,
// `Cache-Control: no-store`:
//
//   GET /api/policy → { maxSecurityFloor: null | SecurityMode, capturePolicy: 'off' | 'protect' }
//
// * `maxSecurityFloor` — the least restrictive rendering mode the reader may use
//   (`viewers/max-security.ts` clamps every mode up to it).
// * `capturePolicy: 'protect'` — ask the platform to keep the window out of
//   screenshots and screen recordings (`platform.setCaptureProtection(true)`).
//   The desktop and Android shells can; a browser cannot, and its implementation
//   answers `{ supported: false }`.
//
// The policy is fetched once per session: the first successful load is kept until
// `resetClientPolicy()`. Before sign-in the route answers 401, which is not an
// error here — the defaults stand and the next `loadClientPolicy()` asks again (at
// most every RETRY_MS, so a reader re-rendering while signed out does not send a
// request per render).

import { createSignal, type Accessor } from 'solid-js';

import { withBase } from './basePath.ts';
import { apiFetch } from './http.ts';
import type { SecurityMode } from '../viewers/sandbox.ts';
import { getPlatform } from '../platform/index.ts';

export type CapturePolicy = 'off' | 'protect';

export interface ClientPolicy {
  maxSecurityFloor: SecurityMode | null;
  capturePolicy: CapturePolicy;
}

/** What applies when no policy has been read: no floor, no capture protection. */
export const DEFAULT_CLIENT_POLICY: ClientPolicy = { maxSecurityFloor: null, capturePolicy: 'off' };

const ENDPOINT = '/api/policy';
const RETRY_MS = 30_000;

const [policy, setPolicy] = createSignal<ClientPolicy>(DEFAULT_CLIENT_POLICY);
let loaded = false;
let inflight: Promise<ClientPolicy> | null = null;
let lastFailure = Number.NEGATIVE_INFINITY;
/** Bumped by `resetClientPolicy`, so a load started before a reset is ignored. */
let generation = 0;

/** The policy in force (reactive). `DEFAULT_CLIENT_POLICY` until a load succeeds. */
export const clientPolicy: Accessor<ClientPolicy> = policy;

function isFloor(v: unknown): v is SecurityMode {
  return v === 'full-sanitized' || v === 'sanitized-no-media' || v === 'plain-text';
}

/** Read the wire shape; an unknown value degrades to the default for that field. */
export function parseClientPolicy(raw: unknown): ClientPolicy {
  const o = (raw ?? {}) as { maxSecurityFloor?: unknown; capturePolicy?: unknown };
  return {
    maxSecurityFloor: isFloor(o.maxSecurityFloor) ? o.maxSecurityFloor : null,
    capturePolicy: o.capturePolicy === 'protect' ? 'protect' : 'off',
  };
}

/** Put a policy into force: the signal, and capture protection when asked for. */
async function apply(p: ClientPolicy, gen: number): Promise<void> {
  if (gen !== generation) return;
  setPolicy(p);
  loaded = true;
  if (p.capturePolicy === 'protect') {
    try {
      await getPlatform().setCaptureProtection(true);
    } catch {
      // The platform could not do it; the reader's watermark still applies.
    }
  }
}

/**
 * Fetch the policy unless this session already has it. Resolves to the policy in
 * force; never rejects. `fetcher` is a test seam.
 */
export function loadClientPolicy(
  fetcher: (input: string, init?: RequestInit) => Promise<Response> = apiFetch,
  now: () => number = Date.now,
): Promise<ClientPolicy> {
  if (loaded) return Promise.resolve(policy());
  if (inflight !== null) return inflight;
  if (now() - lastFailure < RETRY_MS) return Promise.resolve(policy());
  const gen = generation;
  const run = async (): Promise<ClientPolicy> => {
    try {
      const res = await fetcher(withBase(ENDPOINT));
      if (!res.ok) {
        if (gen === generation) lastFailure = now();
        return policy();
      }
      await apply(parseClientPolicy(await res.json()), gen);
      return policy();
    } catch {
      if (gen === generation) lastFailure = now();
      return policy();
    }
  };
  const load: Promise<ClientPolicy> = run().finally(() => {
    if (inflight === load) inflight = null;
  });
  inflight = load;
  return load;
}

/** Forget the session's policy (sign-out, tests). */
export function resetClientPolicy(): void {
  generation += 1;
  setPolicy(DEFAULT_CLIENT_POLICY);
  loaded = false;
  inflight = null;
  lastFailure = Number.NEGATIVE_INFINITY;
}
