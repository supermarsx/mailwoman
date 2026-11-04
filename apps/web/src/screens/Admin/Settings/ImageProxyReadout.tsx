// Image proxy readout, shown above the Image proxy settings (the one section for
// every image cache key): configured against effective values, and per-tier size,
// entries, hit rate, hot and cold counts and promotions per minute. Read from the
// same privacy statistics the Privacy page uses; nothing is invented when the
// server does not report it.

import { createResource, Show, type JSX } from 'solid-js';
import { formatBytes, formatNumber, t } from '../../../i18n';
import { adminRequest } from '../shell/adminApi.ts';

interface TierSide {
  bytes?: number;
  budgetBytes?: number;
  usedBytes?: number;
  entries?: number;
  hot?: number;
  cold?: number;
  hitPercent?: number | null;
  enabled?: boolean;
  active?: boolean;
  path?: string | null;
  lastError?: string | null;
}
export interface ImageTierReadout {
  configured?: {
    disk?: { enabled?: boolean; mode?: string; manualBytes?: number; floorBytes?: number; ceilingBytes?: number };
    ramBytes?: number;
  };
  ram?: TierSide;
  disk?: TierSide;
  index?: { entries?: number };
  promotionsPerMin?: number;
  promotions?: number;
  demotions?: number;
  drops?: number;
}
interface Stats {
  imageCache?: { tiers?: ImageTierReadout };
}

const num = (v: unknown): string => (typeof v === 'number' && Number.isFinite(v) ? formatNumber(v) : t('admin-image-proxy-unknown'));
const bytes = (v: unknown): string => (typeof v === 'number' && Number.isFinite(v) ? formatBytes(v) : t('admin-image-proxy-unknown'));
const pct = (v: unknown): string => (typeof v === 'number' ? `${formatNumber(v)} %` : t('admin-image-proxy-unknown'));

export function ImageProxyReadout(props: { load?: () => Promise<Stats>; /** The screen's own polled statistics: no request of its own, no Refresh button. */ shared?: () => unknown }): JSX.Element {
  const [stats, { refetch }] = createResource<Stats | undefined>(async () => (props.shared ? undefined : await (props.load ?? (async () => (await adminRequest<Stats>('/egress/privacy-stats')).data))()));
  const tiers = (): ImageTierReadout | undefined => (props.shared ? (props.shared() as Stats | undefined)?.imageCache?.tiers : stats.error ? undefined : stats.latest?.imageCache?.tiers);
  return (
    <section data-testid="image-proxy-readout" aria-label={t('admin-image-proxy-title')}>
      <h3>{t('admin-image-proxy-title')}</h3>
      <Show when={tiers()} fallback={<p>{t('admin-image-proxy-unavailable')}</p>}>
        {(r) => (
          <>
            <p data-testid="image-proxy-ram">
              {t('admin-image-proxy-ram', {
                used: bytes(r().ram?.usedBytes),
                budget: bytes(r().ram?.budgetBytes),
                configured: bytes(r().configured?.ramBytes),
                entries: num(r().ram?.entries),
                hot: num(r().ram?.hot),
                cold: num(r().ram?.cold),
                hit: pct(r().ram?.hitPercent),
              })}
            </p>
            <p data-testid="image-proxy-disk">
              <Show
                when={r().disk?.enabled}
                fallback={t('admin-image-proxy-disk-off')}
              >
                {r().disk?.active
                  ? t('admin-image-proxy-disk', {
                      bytes: bytes(r().disk?.bytes),
                      budget: bytes(r().disk?.budgetBytes),
                      mode: r().configured?.disk?.mode ?? '',
                      floor: bytes(r().configured?.disk?.floorBytes),
                      ceiling: bytes(r().configured?.disk?.ceilingBytes),
                      manual: bytes(r().configured?.disk?.manualBytes),
                      entries: num(r().disk?.entries),
                      hot: num(r().disk?.hot),
                      cold: num(r().disk?.cold),
                      hit: pct(r().disk?.hitPercent),
                    })
                  : t('admin-image-proxy-disk-inactive', { reason: r().disk?.lastError ?? t('admin-image-proxy-unknown') })}
              </Show>
            </p>
            <p data-testid="image-proxy-moves">
              {t('admin-image-proxy-moves', {
                perMin: num(r().promotionsPerMin),
                promotions: num(r().promotions),
                demotions: num(r().demotions),
                drops: num(r().drops),
                index: num(r().index?.entries),
              })}
            </p>
          </>
        )}
      </Show>
      <Show when={!props.shared}>
        <button type="button" class="btn" disabled={stats.loading} onClick={() => void refetch()}>
          {t('admin-image-proxy-refresh')}
        </button>
      </Show>
    </section>
  );
}
