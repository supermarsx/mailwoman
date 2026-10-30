// The signed-in mailbox's feature profile (t98): `GET /api/features`, one
// reactive map of catalogue feature id → on/off plus its limits and allowed
// values. The web client hides what is off; the server refuses it regardless
// (`crates/mw-server/src/feature_gate.rs`), so this is presentation only.
//
// Until the map has loaded (and when it cannot load) every feature counts as
// on: a slow or failed read must never hide the mailbox, and the server still
// answers a refused call with `403 {code:"feature_disabled", feature}`.

import { createSignal } from 'solid-js';
import { withBase } from '../api/basePath.ts';
import { apiFetch } from '../api/http.ts';
import { onLinkedMailboxChange } from '../api/linkedMailboxSession.ts';
import { observePreferenceAccount } from '../api/accountPreferenceLifecycle.ts';

export interface FeatureSettingEntry {
  value: unknown;
  source?: string;
  locked?: boolean;
}

export interface FeatureEntry {
  enabled: boolean;
  category?: string;
  limits: Record<string, FeatureSettingEntry>;
  settings: Record<string, FeatureSettingEntry>;
}

export type FeatureMap = Record<string, FeatureEntry>;

type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const [map, setMap] = createSignal<FeatureMap>({});
const [loaded, setLoaded] = createSignal(false);
let generation = 0;
let inflight: Promise<void> | null = null;

function parse(body: unknown): FeatureMap {
  const out: FeatureMap = {};
  const features = (body as { features?: unknown } | null)?.features;
  if (features === null || typeof features !== 'object') return out;
  for (const [id, raw] of Object.entries(features as Record<string, unknown>)) {
    if (raw === null || typeof raw !== 'object') continue;
    const r = raw as Record<string, unknown>;
    if (typeof r['enabled'] !== 'boolean') continue;
    const entries = (v: unknown): Record<string, FeatureSettingEntry> =>
      v !== null && typeof v === 'object' ? (v as Record<string, FeatureSettingEntry>) : {};
    out[id] = {
      enabled: r['enabled'],
      ...(typeof r['category'] === 'string' ? { category: r['category'] } : {}),
      limits: entries(r['limits']),
      settings: entries(r['settings']),
    };
  }
  return out;
}

/** Load (or reload) the map. A failure keeps the last answer. */
export function loadFeatures(opts: { refresh?: boolean; fetcher?: Fetcher } = {}): Promise<void> {
  if (inflight !== null && opts.refresh !== true) return inflight;
  const mine = ++generation;
  const fetcher: Fetcher = opts.fetcher ?? ((input, init) => apiFetch(input, init));
  let run: Promise<void> | null = null;
  run = (async () => {
    try {
      const res = await fetcher(withBase('/api/features'), { headers: { accept: 'application/json' } });
      if (!res.ok) return;
      const next = parse(await res.json());
      if (mine !== generation) return;
      setMap(next);
      setLoaded(true);
    } catch {
      /* keep what we had: the server still refuses what is off */
    } finally {
      if (inflight === run) inflight = null;
    }
  })();
  inflight = run;
  return run;
}

/** Forget the map (sign-out, account change). */
export function clearFeatures(): void {
  generation++;
  inflight = null;
  setMap({});
  setLoaded(false);
}

/** Whether the map has an answer from the server (reactive). */
export function featuresLoaded(): boolean {
  return loaded();
}

/** Whether `id` is on for this mailbox (reactive). Unknown or not loaded = on. */
export function featureOn(id: string): boolean {
  return map()[id]?.enabled ?? true;
}

/** A numeric limit (`limit.*`) of a feature; `null` = none (0 or not set). */
export function featureLimit(id: string, key: string): number | null {
  const v = map()[id]?.limits[key]?.value;
  return typeof v === 'number' && v > 0 ? v : null;
}

/** The whole map (reactive), for tests and diagnostics. */
export function features(): FeatureMap {
  return map();
}

/** Whether an API error body is a feature refusal (`403 feature_disabled`). */
export function isFeatureRefusal(body: unknown): body is { code: 'feature_disabled'; feature: string } {
  return body !== null && typeof body === 'object' && (body as { code?: unknown }).code === 'feature_disabled';
}

// Signed-in account changes (sign-in, sign-out, another account): start over.
// A linked mailbox has its own profile: reload when the active mailbox changes.
if (typeof window !== 'undefined') {
  observePreferenceAccount((account) => {
    clearFeatures();
    if (account) void loadFeatures({ refresh: true });
  });
  onLinkedMailboxChange(() => {
    if (loaded() || inflight !== null) {
      clearFeatures();
      void loadFeatures({ refresh: true });
    }
  });
}
