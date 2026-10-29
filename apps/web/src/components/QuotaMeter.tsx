// Storage usage (t39-e7, L10): a compact meter under the folder list and a
// detail table in Settings > Folders. Both stay empty when the server reports
// no quota. Warnings follow the administrator's `quota.warn*` settings.

import { createResource, For, onMount, Show, type JSX } from 'solid-js';
import { createClient } from '../api/client.ts';
import { settings } from '../api/settings.ts';
import { loadQuotas, ratio, storageQuota, warnLevel, warnPolicy, type Quota } from '../api/quota.ts';
import { formatBytes, formatNumber, formatPercent, loadCatalog, t } from '../i18n';
import * as css from './quotaMeter.css.ts';
import { resourceValue } from './resourceValue.ts';

export type QuotaLoader = (accountId: string) => Promise<Quota[]>;

let sharedClient: ReturnType<typeof createClient> | null = null;
const defaultLoader: QuotaLoader = (accountId) => loadQuotas((sharedClient ??= createClient()), accountId);

export interface QuotaProps {
  accountId: string | null;
  /** Read again whenever this changes (e.g. the folder list after a sync). */
  refresh?: unknown;
  load?: QuotaLoader;
}

function useQuotas(props: QuotaProps) {
  onMount(() => void loadCatalog('quota'));
  const [quotas] = createResource(
    () => (props.accountId ? { accountId: props.accountId, refresh: props.refresh } : null),
    // A failed read shows nothing: the meter is informational.
    async ({ accountId }) => (props.load ?? defaultLoader)(accountId).catch(() => [] as Quota[]),
  );
  return quotas;
}

function amount(q: Quota, value: number): string {
  return q.resourceType === 'octets' ? formatBytes(value) : formatNumber(value);
}

/** The sidebar meter: the fullest storage quota, or nothing. */
export function QuotaMeter(props: QuotaProps): JSX.Element {
  const quotas = useQuotas(props);
  const storage = () => storageQuota(resourceValue(quotas) ?? []);
  return <Show when={storage()}>{(q) => {
    const warning = () => warnLevel(q(), warnPolicy(settings()));
    const text = () => t('quota-meter-label', { used: amount(q(), q().used), limit: amount(q(), q().hardLimit) });
    return <div class={css.meter} data-testid="quota-meter" data-level={warning()?.level ?? 'ok'}>
      <span class={css.label} title={text()}>{text()}</span>
      <div class={css.track} role="meter" aria-label={t('quota-meter-name')} aria-valuemin={0} aria-valuemax={100}
        aria-valuenow={Math.round(Math.min(1, ratio(q())) * 100)} aria-valuetext={text()}>
        <div class={css.fill[warning()?.level ?? 'ok']} style={{ width: `${Math.min(100, ratio(q()) * 100)}%` }} />
      </div>
      <Show when={warning()}>{(w) => <span class={css.warning} role="status">
        {t(w().level === 'critical' ? 'quota-warning-critical' : 'quota-warning', { percent: formatPercent(Math.floor(ratio(q()) * 100) / 100, { maximumFractionDigits: 0 }) })}
      </span>}</Show>
    </div>;
  }}</Show>;
}

/** Settings detail: every reported quota with its source; nothing without one. */
export function QuotaDetails(props: QuotaProps): JSX.Element {
  const quotas = useQuotas(props);
  const list = () => resourceValue(quotas) ?? [];
  return <Show when={list().length > 0}>
    <section aria-labelledby="quota-details-heading" data-testid="quota-details">
      <h3 id="quota-details-heading">{t('quota-details-title')}</h3>
      <p>{t('quota-details-help')}</p>
      <table class={css.table}>
        <thead><tr><th scope="col">{t('quota-details-resource')}</th><th scope="col">{t('quota-details-usage')}</th><th scope="col">{t('quota-details-source')}</th></tr></thead>
        <tbody><For each={list()}>{(q) => {
          const warning = () => warnLevel(q, warnPolicy(settings()));
          const percent = () => formatPercent(Math.floor(ratio(q) * 100) / 100, { maximumFractionDigits: 0 });
          return <tr>
            <th scope="row">{t(q.resourceType === 'octets' ? 'quota-resource-storage' : 'quota-resource-messages')}</th>
            <td>
              {t('quota-details-amount', { used: amount(q, q.used), limit: amount(q, q.hardLimit), percent: percent() })}
              <div class={css.detailTrack} aria-hidden="true"><div class={css.fill[warning()?.level ?? 'ok']} style={{ width: `${Math.min(100, ratio(q) * 100)}%` }} /></div>
              <Show when={warning()}>{(w) => <p class={css.warning} role="status">{t(w().level === 'critical' ? 'quota-warning-critical' : 'quota-warning', { percent: percent() })}</p>}</Show>
            </td>
            <td>{t(q.id.startsWith('mail-') ? 'quota-source-admin' : 'quota-source-server')}</td>
          </tr>;
        }}</For></tbody>
      </table>
    </section>
  </Show>;
}
