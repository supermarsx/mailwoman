import type { RetentionStore } from '../../../api/admin/retention.ts';

/** The store ids the console knows a name and description for. */
export const storeLabel = (id: string): string => `admin-retention-store-${id}`;
export const storeData = (id: string): string => `admin-retention-data-${id}`;

/** Which message describes a store's limit, with its arguments. */
export function limitMessage(s: RetentionStore): { id: string; args: Record<string, number> } {
  if (s.fixed && s.days !== null) return { id: 'admin-retention-limit-fixed', args: { days: s.days } };
  if (s.days === null) return { id: 'admin-retention-limit-policy', args: {} };
  if (s.unit === 'grace') return { id: 'admin-retention-limit-grace', args: { days: s.days } };
  if (s.days === 0) return { id: 'admin-retention-limit-forever', args: {} };
  if (s.unit === 'ip') return { id: 'admin-retention-limit-ip', args: { days: s.days } };
  return { id: 'admin-retention-limit-days', args: { days: s.days } };
}

/** A sweeper store whose limit of 0 keeps its rows: the console warns. */
export const keepsForever = (s: RetentionStore): boolean => s.owner === 'sweeper' && s.unit !== 'grace' && s.days === 0;

/** The distinct settings that limits are edited through, in table order. */
export function limitKeys(stores: readonly RetentionStore[]): string[] {
  const seen = new Set<string>();
  for (const s of stores) if (s.key && s.owner === 'sweeper') seen.add(s.key);
  return [...seen];
}

/** Stores that have a number to plot, largest first. */
export function topBy(stores: readonly RetentionStore[], pick: (s: RetentionStore) => number | null, limit = 12): { id: string; value: number }[] {
  return stores
    .map((s) => ({ id: s.id, value: pick(s) ?? 0 }))
    .filter((x) => x.value > 0)
    .sort((a, b) => b.value - a.value)
    .slice(0, limit);
}

/** Labels of the age buckets for the given edges (days): younger than the first edge, ..., older than the last. */
export function bucketLabels(edges: readonly number[], translate: (id: string, args?: Record<string, number>) => string): string[] {
  const out = edges.map((e, i) => (i === 0
    ? translate('admin-retention-age-under', { days: e })
    : translate('admin-retention-age-between', { from: edges[i - 1] ?? 0, to: e })));
  out.push(translate('admin-retention-age-over', { days: edges[edges.length - 1] ?? 0 }));
  return out;
}

/** Message ids for the cross-check warnings of the class settings. */
export function classWarnings(c: { warnings: readonly string[] }): string[] {
  return c.warnings.map((w) => `admin-retention-warn-${w}`);
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ['KiB', 'MiB', 'GiB', 'TiB'];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(v < 10 ? 1 : 0)} ${units[i]}`;
}
