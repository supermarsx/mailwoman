// Settings > Privacy & data > What we hold about you: the stored data by plain
// category with counts, sizes, retention and processors; the purposes you can
// consent to or withdraw from; automated-processing notice and per-feature
// opt-out; your restriction status; sessions and app keys you can revoke; and
// shortcuts into the data request form. The data request part below stays as is.
import { createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { formatBytes, formatDateTime, formatNumber } from '../../i18n/format.ts';
import {
  decideConsent, listKeys, listSessions, loadOverview, revokeKey, revokeSession,
  type ConsentPurpose, type ConsentView, type DataCategory, type KeyRow, type PrivacyOverview, type SessionRow,
} from '../../api/privacy.ts';
import { chooseRequest } from '../../api/privacyShortcut.ts';
import { withBase } from '../../api/basePath.ts';
import * as css from './styles.css.ts';

const when = (seconds: number | null): string => (seconds === null ? '' : formatDateTime(seconds * 1000));

function Categories(props: { items: DataCategory[] }): JSX.Element {
  return <div role="region" aria-label={t('yd-categories-title')} tabindex="0">
    <table data-testid="yd-categories">
      <caption class={css.meta}>{t('yd-categories-help')}</caption>
      <thead><tr>
        <th scope="col">{t('yd-col-category')}</th><th scope="col">{t('yd-col-count')}</th><th scope="col">{t('yd-col-size')}</th>
        <th scope="col">{t('yd-col-retention')}</th><th scope="col">{t('yd-col-processors')}</th>
      </tr></thead>
      <tbody><For each={props.items}>{c => <tr data-testid={`yd-cat-${c.id}`}>
        <th scope="row">{t(`yd-cat-${c.id}`)}</th>
        <td>{formatNumber(c.rows)}</td>
        <td>{c.bytes > 0 ? t('yd-approx', { size: formatBytes(c.bytes) }) : t('yd-size-none')}</td>
        <td>{c.retention ? t(c.retention.source === 'userRule' ? 'yd-retention-rule' : 'yd-retention-days', { days: c.retention.days }) : t('yd-retention-none')}</td>
        <td>{c.processors.length > 0 ? c.processors.map(p => t(`yd-proc-${p}`)).join(', ') : t('yd-proc-none')}</td>
      </tr>}</For></tbody>
    </table>
  </div>;
}

function Purpose(props: { purpose: ConsentPurpose; busy: boolean; onDecide: (decision: 'granted' | 'withdrawn') => void }): JSX.Element {
  const p = (): ConsentPurpose => props.purpose;
  const on = (): boolean => p().state === 'granted' || (p().state === 'unset' && p().basis !== 'consent' && p().canToggle);
  return <li class={css.item} data-testid={`yd-purpose-${p().id}`}>
    <div class={css.itemMain}>
      <span class={css.itemName}>{t(`yd-purpose-${p().id}-name`)} <span class={css.badge}>{t(`yd-basis-${p().basis}`)}</span></span>
      <span class={css.meta}>{p().text ?? t(`yd-purpose-${p().id}-desc`)}</span>
      <Show when={p().state === 'stale'}><span class={css.meta} role="status">{t('yd-consent-stale')}</span></Show>
      <Show when={p().decidedAt}><span class={css.meta}>{t('yd-consent-decided', { state: t(`yd-state-${p().state}`), date: when(p().decidedAt) })}</span></Show>
      <Show when={!p().canToggle}><span class={css.meta}>{t(`yd-managed-${p().enforcement}`)}</span></Show>
    </div>
    <Show when={p().canToggle}>
      <label class={css.check}>
        <input type="checkbox" checked={on()} disabled={props.busy}
          onChange={e => props.onDecide(e.currentTarget.checked ? 'granted' : 'withdrawn')} />
        {' '}{on() ? t('yd-consent-on') : t('yd-consent-off')}
      </label>
    </Show>
  </li>;
}

function Consents(props: { view: ConsentView; onChange: (v: ConsentView) => void }): JSX.Element {
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  async function decide(id: string, decision: 'granted' | 'withdrawn'): Promise<void> {
    setBusy(true); setError('');
    try { props.onChange(await decideConsent(id, decision)); } catch { setError(t('yd-error')); } finally { setBusy(false); }
  }
  return <>
    <h4 class={css.subheading}>{t('yd-consents-title')}</h4>
    <p class={css.meta}>{t('yd-consents-help')}</p>
    <ul class={css.list} data-testid="yd-consents"><For each={props.view.purposes}>{p => <Purpose purpose={p} busy={busy()} onDecide={d => void decide(p.id, d)} />}</For></ul>
    <Show when={props.view.art22.enabled}>
      <section data-testid="yd-art22" aria-labelledby="yd-art22-title">
        <h4 id="yd-art22-title" class={css.subheading}>{t('yd-art22-title')}</h4>
        <p class={css.prose}>{t('yd-art22-logic')}</p>
        <p class={css.prose}>{t('yd-art22-significance')}</p>
        <p class={css.prose}>{t('yd-art22-review')}</p>
        <ul class={css.list}><For each={props.view.assistantFeatures}>{f => <li class={css.item} data-testid={`yd-feature-${f.feature}`}>
          <div class={css.itemMain}><span class={css.itemName}>{t(`yd-feature-${f.feature}`)}</span><span class={css.meta}>{t(`yd-feature-${f.feature}-desc`)}</span></div>
          <label class={css.check}><input type="checkbox" checked={f.enabled} disabled={busy()} onChange={e => void decide(f.id, e.currentTarget.checked ? 'granted' : 'withdrawn')} /> {f.enabled ? t('yd-feature-on') : t('yd-feature-off')}</label>
        </li>}</For></ul>
      </section>
    </Show>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <details>
      <summary>{t('yd-receipts-title', { count: props.view.receipts.length })}</summary>
      <ol class={css.meta} data-testid="yd-receipts"><For each={props.view.receipts}>{r => <li>{when(r.createdAt)} · {r.purpose} · {t(`yd-state-${r.decision}`)} · v{r.version} · {r.source}</li>}</For></ol>
    </details>
  </>;
}

function Devices(): JSX.Element {
  const [sessions, { refetch }] = createResource(async () => { try { return await listSessions(); } catch { return [] as SessionRow[]; } });
  const [keys, { refetch: refetchKeys }] = createResource(async () => { try { return await listKeys(); } catch { return [] as KeyRow[]; } });
  const [error, setError] = createSignal('');
  async function run(work: () => Promise<void>, reload: () => unknown): Promise<void> {
    setError('');
    try { await work(); await reload(); } catch { setError(t('yd-error')); }
  }
  return <>
    <h4 class={css.subheading}>{t('yd-sessions-title')}</h4>
    <Show when={(sessions() ?? []).length > 0} fallback={<p class={css.meta}>{t('yd-sessions-none')}</p>}>
      <ul class={css.list} data-testid="yd-sessions"><For each={sessions() ?? []}>{s => <li class={css.item}>
        <div class={css.itemMain}>
          <span class={css.itemName}>{s.deviceLabel || s.userAgent || t('yd-session-unknown')}{s.current ? <> <span class={css.badge}>{t('yd-session-current')}</span></> : null}</span>
          <span class={css.meta}>{s.lastSeen ?? ''}{s.country ? ` · ${s.country}` : ''}</span>
        </div>
        <Show when={!s.current}><button type="button" class={css.ghost} onClick={() => void run(() => revokeSession(s.handle), refetch)}>{t('yd-revoke')}</button></Show>
      </li>}</For></ul>
    </Show>
    <h4 class={css.subheading}>{t('yd-keys-title')}</h4>
    <Show when={(keys() ?? []).length > 0} fallback={<p class={css.meta}>{t('yd-keys-none')}</p>}>
      <ul class={css.list} data-testid="yd-keys"><For each={keys() ?? []}>{k => <li class={css.item}>
        <div class={css.itemMain}><span class={css.itemName}>{k.label || k.name || k.prefix}</span><span class={css.meta}>{k.prefix}</span></div>
        <button type="button" class={css.ghost} onClick={() => void run(() => revokeKey(k.prefix), refetchKeys)}>{t('yd-revoke')}</button>
      </li>}</For></ul>
    </Show>
    <Show when={error()}><p role="alert">{error()}</p></Show>
  </>;
}

export function YourData(props: { load?: () => Promise<PrivacyOverview> } = {}): JSX.Element {
  onMount(() => { void loadCatalog('your-data'); });
  const [overview, { mutate }] = createResource(async () => { try { return await (props.load ?? loadOverview)(); } catch { return null; } });
  return <section class={css.section} aria-labelledby="your-data-heading" data-testid="your-data">
    <h3 id="your-data-heading" class={css.heading}>{t('yd-title')}</h3>
    <Show when={overview()} fallback={<p class={css.meta}>{overview.loading ? t('yd-loading') : t('yd-unavailable')}</p>}>{o => <>
      <p class={css.prose}>{t('yd-intro', { address: o().account.address })}</p>
      <Show when={o().restriction.active}>
        <p class={css.warn} role="status" data-testid="yd-restriction">
          {t('yd-restriction-active', { scopes: o().restriction.scopes.map(s => t(`yd-scope-${s}`)).join(', '), date: when(o().restriction.since) })}
          <Show when={o().restriction.expiresAt}> {t('yd-restriction-until', { date: when(o().restriction.expiresAt) })}</Show>
        </p>
      </Show>
      <Categories items={o().categories} />
      <Show when={o().consents.enabled}><Consents view={o().consents} onChange={v => mutate({ ...o(), consents: v })} /></Show>
      <Devices />
      <h4 class={css.subheading}>{t('yd-shortcuts-title')}</h4>
      <div class={css.actions}>
        <button type="button" class={css.button} onClick={() => chooseRequest('portability')}>{t('yd-download')}</button>
        <button type="button" class={css.ghost} onClick={() => chooseRequest('restriction')}>{t('yd-restrict')}</button>
        <button type="button" class={css.danger} onClick={() => chooseRequest('erasure')}>{t('yd-delete')}</button>
      </div>
      <p class={css.meta}>
        <a href={withBase(o().links.policy)} rel="noopener">{t('yd-link-policy')}</a>{' · '}
        <a href={withBase(o().links.processing)} rel="noopener">{t('yd-link-processing')}</a>
      </p>
      <p class={css.meta}>{t('yd-prefs-pointer')}</p>
    </>}</Show>
  </section>;
}
