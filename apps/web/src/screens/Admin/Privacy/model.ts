// Shared types and helpers of the Privacy screen and its tabs.

import { durationFormatterFor } from '../../../i18n';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import type { ImageConcurrencyMode, ImageConcurrencyRuntime, ImageRateRuntime, ImageCacheRuntime } from '../AdminImageConcurrency.tsx';
import { IMAGE_CONCURRENCY_KEY } from '../AdminImageConcurrency.tsx';

export const HISTORY_RANGES = ['15m', '1h', '6h', '24h'] as const;
export type HistoryRange = (typeof HISTORY_RANGES)[number];

/** `history` of the privacy statistics: in-memory on the server, empty after a restart. */
export interface ProxyHistory {
  range?: string;
  rangeSecs?: number;
  stepSecs?: number;
  sampleSecs?: number;
  retentionSecs?: number;
  resetsOnRestart?: boolean;
  samples?: number;
  now?: number;
  t: number[];
  series: Record<string, (number | null)[]>;
  classes?: Record<string, { allowed: (number | null)[]; refused: (number | null)[]; failed: (number | null)[] }>;
}

/** One summarised histogram of the proxy's speed; times in milliseconds, `null` when nothing was measured. */
export interface LatencySummary { count: number; p50Ms: number | null; p95Ms: number | null; p99Ms: number | null; maxMs: number | null; meanMs: number | null }
export type SpeedVerdict = 'healthy' | 'slow' | 'degraded' | 'noData';

/** `latency` of the privacy statistics: per-process, in memory, empty after a restart. */
export interface ProxyLatency {
  enabled: boolean;
  scope?: string;
  resetsOnRestart?: boolean;
  sliceSecs?: number;
  stepSecs?: number;
  relativeError?: number;
  range?: string;
  t: number[];
  series: Record<string, (number | null)[]>;
  total: LatencySummary;
  hit: LatencySummary;
  miss: LatencySummary;
  outcomes: Record<string, LatencySummary>;
  phases: Record<string, LatencySummary & { sharePct: number | null }>;
  otherPct: number | null;
  failures: number;
  failurePct: number | null;
  health: { verdict: SpeedVerdict; reason: string; thresholds: Record<string, number> };
}

export interface PrivacyStats {
  imageConcurrency?: ImageConcurrencyRuntime; imageRate?: ImageRateRuntime; imageCache?: ImageCacheRuntime;
  cacheEntries: number; cacheBytes: number; trackersReplaced: number;
  traffic: { class: string; allowed: number; refused: number; failed: number; totalMs: number; /** Exact total in μs (servers from 26.20). */ totalUs?: number }[];
  history?: ProxyHistory;
  /** Image proxy speed (newer servers only; absent on older ones). */
  latency?: ProxyLatency;
}
export type TrafficRow = PrivacyStats['traffic'][number];

/** t56: the remote-image fetch limits, shown in their own tab under the console's registry labels. Milliseconds read as seconds. */
export const FETCH_LIMIT_KEYS: readonly string[] = ['privacy.fetchConnectTimeoutMs','privacy.fetchHeadersTimeoutMs','privacy.fetchIdleTimeoutMs','privacy.fetchTotalTimeoutMs','privacy.fetchQueueTimeoutMs','privacy.imageDecodeTimeoutMs','privacy.maxRedirects','privacy.maxConcurrentPerMessage','privacy.maxConcurrentPerAccount','privacy.maxConcurrentGlobal','privacy.maxConcurrentPipeline','privacy.maxConcurrentDecodes','privacy.maxQueuedRequests','privacy.ratePerMin','privacy.rateBurst','privacy.maxBytes','privacy.cacheBytes','privacy.cacheTtlSecs','privacy.negativeCacheTtlSecs','privacy.imageRetryAttempts','privacy.imageRetryDelayMs'];

/** The remote-content and tracker policy keys, in the order the tab shows them. */
export const REMOTE_KEYS: readonly string[] = ['privacy.remoteContent','privacy.remoteImageDownloads','privacy.remoteImagesToast','privacy.trackers','privacy.trackerOverrideAllowed','privacy.cleanLinks','privacy.allowDirect','privacy.stripMailerHeaders'];

export type PrivacyTabId = 'overview' | 'remote' | 'image-proxy' | 'fetch-limits' | 'traffic';

/** Which tab edits a key; the Image proxy tab takes everything of the `imageProxy` category. */
export function tabForKey(key: string, rows: readonly AdminSetting[] = []): PrivacyTabId | undefined {
  if (FETCH_LIMIT_KEYS.includes(key)) return 'fetch-limits';
  if (key === IMAGE_CONCURRENCY_KEY || key.startsWith('privacy.image')) return 'image-proxy';
  if (rows.find((r) => r.key === key)?.category === 'imageProxy') return 'image-proxy';
  if (REMOTE_KEYS.includes(key)) return 'remote';
  return undefined;
}

/** The settings of the Image proxy tab: the registry category, minus the fetch-limit keys and the mode card's key. */
export function imageProxyRows(rows: readonly AdminSetting[]): AdminSetting[] {
  return rows.filter((r) => r.category === 'imageProxy' && !FETCH_LIMIT_KEYS.includes(r.key) && r.key !== IMAGE_CONCURRENCY_KEY);
}

export type ImageGroup = 'rate' | 'cache' | 'ram' | 'disk' | 'tiering';
/** Subsection of an Image proxy key. */
export function imageGroup(key: string): ImageGroup {
  if (/^privacy\.imageRate/.test(key)) return 'rate';
  if (/^privacy\.imageDisk/.test(key)) return 'disk';
  if (/^privacy\.imageRam/.test(key)) return 'ram';
  if (/^privacy\.(imageCache|cache)/.test(key)) return 'cache';
  return 'tiering';
}

export const fetchLabel = (key: string): string => `admin-settings-key-${key.replace('.', '-').replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase()}`;

const trafficMs = (row: TrafficRow): number => typeof row.totalUs === 'number' ? row.totalUs / 1000 : row.totalMs;
/** Total time per class, in one unit for the whole column. Whole-ms totals below 1 ms read "<1 ms". */
export const trafficTime = (rows: readonly TrafficRow[]): ((row: TrafficRow) => string) => {
  const fmt = durationFormatterFor(rows.map(trafficMs));
  return (row) => row.allowed + row.refused + row.failed === 0 ? fmt(0) : fmt(trafficMs(row), typeof row.totalUs === 'number' ? 0.001 : 1);
};

/** `5000` ms -> "5 s" in the document's language; counts stay plain numbers. */
export function fetchLimitValue(key: string, value: unknown, lang = document.documentElement.lang || undefined): string {
  if (typeof value !== 'number') return String(value);
  if (!key.endsWith('Ms') && !key.endsWith('Secs')) return new Intl.NumberFormat(lang).format(value);
  return new Intl.NumberFormat(lang, { style: 'unit', unit: 'second', unitDisplay: 'short', maximumFractionDigits: 1 }).format(key.endsWith('Ms') ? value / 1000 : value);
}

/** What every tab receives from the screen. */
export interface PrivacyTabContext {
  rows: () => AdminSetting[];
  settingsError: () => unknown;
  canReadSettings: () => boolean;
  stats: () => PrivacyStats | undefined;
  statsError: () => unknown;
  statsLoading: () => boolean;
  updatedAt: () => number | null;
  range: () => HistoryRange;
  setRange: (r: HistoryRange) => void;
  refresh: () => void;
  /** The key a deep link names, to focus. */
  focusKey: () => string | undefined;
  onDirty: (key: string, dirty: boolean) => void;
  onSaved: (row: AdminSetting) => void;
  reloadSettings: () => void;
  saveMode?: ((mode: ImageConcurrencyMode, etag: string) => Promise<AdminSetting>) | undefined;
}
