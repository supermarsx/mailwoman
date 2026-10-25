import { createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';
import { loadCatalog, t } from '../../i18n';
import * as css from './styles.css.ts';
import { TwoFactor } from './TwoFactor.tsx';
import { Sessions } from './Sessions.tsx';
import { AsyncBoundary } from '../../components/ErrorBoundary.tsx';

type History = { id: string; ts: number; method: string; result: string; ip: string; userAgent: string };
type Activity = { id: string; timestamp: string; action: string };
type Device = { id: string; label: string; expiresAt: number };
async function request<T>(path: string, body?: object): Promise<T> {
  const response = await apiFetch(`${basePath()}/api/account/security/${path}`, {
    credentials: 'same-origin', ...(body ? { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) } : {}),
  });
  if (!response.ok) throw new Error(t('security-center-error'));
  return await response.json() as T;
}
export function SecurityCenter(): JSX.Element {
  onMount(() => void loadCatalog('security'));
  const [filter, setFilter] = createSignal('');
  const [history, { refetch: refreshHistory }] = createResource(filter, (result) => request<{ events: History[] }>(`history?limit=100${result ? `&result=${result}` : ''}`));
  const [activity] = createResource(() => request<{ events: Activity[] }>('activity'));
  const [devices, { refetch: refreshDevices }] = createResource(() => request<{ devices: Device[] }>('remembered-devices'));
  const [label, setLabel] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  async function action(run: () => Promise<unknown>): Promise<void> {
    setBusy(true); setError('');
    try { await run(); await refreshDevices(); } catch (e) { setError(e instanceof Error ? e.message : t('security-center-error')); }
    finally { setBusy(false); }
  }
  async function exportHistory(): Promise<void> {
    const data = await request<{ events: History[] }>('history?export=true&limit=500');
    const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }));
    const anchor = document.createElement('a'); anchor.href = url; anchor.download = 'signin-history.json'; anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  return <section class={css.section} aria-label={t('security-center-title')}>
    <h2 class={css.heading}>{t('security-center-title')}</h2>
    <p class={css.prose}>{t('security-center-passkey-mode')}</p>
    <AsyncBoundary><TwoFactor /></AsyncBoundary>
    <AsyncBoundary><Sessions /></AsyncBoundary>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <h3 class={css.subheading}>{t('security-center-history')}</h3>
    <label class={css.field}>{t('security-center-result')}<select class={css.input} value={filter()} onChange={(e) => setFilter(e.currentTarget.value)}>
      <option value="">{t('security-center-all')}</option><option value="success">{t('security-center-success')}</option><option value="failure">{t('security-center-failure')}</option><option value="blocked">{t('security-center-blocked')}</option>
    </select></label>
    <div class={css.actions}><button type="button" disabled={busy()} onClick={() => void action(async () => { await refreshHistory(); })}>{t('security-center-refresh')}</button><button type="button" disabled={busy()} onClick={() => void action(exportHistory)}>{t('security-center-export')}</button></div>
    <ul class={css.list}><For each={history()?.events}>{(event) => <li class={css.item}><div><time>{new Date(event.ts * 1000).toLocaleString()}</time> · {event.result} · {event.method}<p class={css.meta}>{event.ip} · {event.userAgent}</p></div></li>}</For></ul>
    <h3 class={css.subheading}>{t('security-center-devices')}</h3>
    <p class={css.prose}>{t('security-center-remember-help')}</p>
    <form onSubmit={(e) => { e.preventDefault(); void action(async () => { await request('remembered-devices', { label: label() }); setLabel(''); }); }}>
      <label class={css.field}>{t('security-center-label')}<input class={css.input} maxLength={128} required value={label()} onInput={(e) => setLabel(e.currentTarget.value)} /></label>
      <button type="submit" disabled={busy()}>{t('security-center-remember')}</button>
    </form>
    <ul class={css.list}><For each={devices()?.devices}>{(device) => <li class={css.item}><span>{device.label} · {new Date(device.expiresAt * 1000).toLocaleDateString()}</span><button type="button" disabled={busy()} onClick={() => void action(() => request('remembered-devices/revoke', { id: device.id }))}>{t('security-center-revoke')}</button></li>}</For></ul>
    <h3 class={css.subheading}>{t('security-center-activity')}</h3>
    <ul class={css.list}><For each={activity()?.events}>{(event) => <li class={css.item}>{event.timestamp} · {event.action}</li>}</For></ul>
  </section>;
}
