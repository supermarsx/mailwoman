// `GET /admin/dashboard/health` client (t103): the Health panel's probes.
//
// Every component has the same shape. `disabled` is a deployment choice and
// comes with `enable` (how to turn it on); a probe that did not answer is
// `fail` with the reason. The panel never shows a component as "not reported".

import { basePath } from '../basePath.ts';
import { adminUrl } from '../adminPath.ts';
import { apiFetch } from '../http.ts';

export type HealthStatus = 'ok' | 'warn' | 'fail' | 'disabled';

export type HealthComponentId = 'database' | 'migrations' | 'sync' | 'outbox' | 'search' | 'render' | 'plugins';

export const HEALTH_COMPONENTS: readonly HealthComponentId[] = ['database', 'migrations', 'sync', 'outbox', 'search', 'render', 'plugins'];

export interface HealthComponent {
  status: HealthStatus;
  /** One sentence from the server (English). */
  reason: string;
  /** Disabled only: how to turn it on. */
  enable?: string;
  metrics: Record<string, unknown>;
  thresholds: Record<string, number>;
  /** Why a metric is null, by metric key. */
  notes?: Record<string, string>;
  /** How long the probe took, in μs. */
  probeUs?: number;
}

export interface HealthSample {
  t: number;
  dbPingMs: number | null;
  renderPingMs: number | null;
  syncErrorsLastHour: number | null;
  outboxQueued: number | null;
  outboxDeferred: number | null;
  searchPending: number | null;
}

export interface DashboardHealth {
  /** Kept for older consoles: the database round trip in ms. */
  dbPingMs: number;
  migration: { applied: number; expected: number };
  /** When the probes ran (Unix seconds); results are reused for `ttlSecs`. */
  checkedAt: number;
  ageSecs: number;
  ttlSecs: number;
  components: Record<HealthComponentId, HealthComponent>;
  history: HealthSample[];
  historyStepSecs: number;
}

export type HealthResult = { kind: 'ok'; data: DashboardHealth } | { kind: 'error'; status: number | null };

export type HealthLoader = (fresh: boolean) => Promise<HealthResult>;

/** The production loader: `GET <base>/admin/dashboard/health[?fresh=1]`. */
export function createHealthLoader(base = basePath()): HealthLoader {
  return async (fresh) => {
    let res: Response;
    try {
      res = await apiFetch(adminUrl(fresh ? '/dashboard/health?fresh=1' : '/dashboard/health', base), { credentials: 'same-origin' });
    } catch {
      return { kind: 'error', status: null };
    }
    if (!res.ok) return { kind: 'error', status: res.status };
    try {
      return { kind: 'ok', data: (await res.json()) as DashboardHealth };
    } catch {
      return { kind: 'error', status: res.status };
    }
  };
}

/** A numeric metric, or null when absent or not a finite number. */
export function num(m: Record<string, unknown> | undefined, key: string): number | null {
  const v = m?.[key];
  return typeof v === 'number' && Number.isFinite(v) ? v : null;
}

/** A string metric, or null. */
export function str(m: Record<string, unknown> | undefined, key: string): string | null {
  const v = m?.[key];
  return typeof v === 'string' ? v : null;
}

/** A nested object metric, or an empty object. */
export function obj(m: Record<string, unknown> | undefined, key: string): Record<string, unknown> {
  const v = m?.[key];
  return v !== null && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : {};
}

/** An array metric of objects, or an empty array. */
export function list(m: Record<string, unknown> | undefined, key: string): Record<string, unknown>[] {
  const v = m?.[key];
  return Array.isArray(v) ? (v.filter((x) => x !== null && typeof x === 'object') as Record<string, unknown>[]) : [];
}
