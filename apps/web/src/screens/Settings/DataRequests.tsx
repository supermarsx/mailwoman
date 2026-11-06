// Settings > Privacy & data > Your data: file and follow data requests (access,
// download, correction, erasure, restriction, objection, other). What is offered,
// the approval steps and the grace period are the administrator's `dsr.*` settings.
import { createEffect, createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { activeLocaleSignal as locale } from '../../i18n/registry.ts';
import { formatDateTime } from '../../i18n/format.ts';
import {
  REQUEST_KINDS, cancelRequest, confirmRequest, downloadLink, listRequests, loadOverview, loadRequest, submitRequest,
  type DataRequest, type DataRequestPolicy, type RequestKind, type TrailEntry,
} from '../../api/dataRequests.ts';
import { requestedKind } from '../../api/privacyShortcut.ts';
import * as css from './styles.css.ts';

const when = (seconds: number): string => formatDateTime(seconds * 1000);
const EXPORT_KINDS: readonly RequestKind[] = ['access', 'portability'];

function introText(policy: DataRequestPolicy, language: string): string {
  const text = policy.text.intro;
  return text[language] ?? text['en'] ?? Object.values(text)[0] ?? t('dr-intro-default');
}

function Contact(props: { policy: DataRequestPolicy }): JSX.Element {
  const c = (): DataRequestPolicy['contact'] => props.policy.contact;
  return <Show when={c().name || c().email || c().url}>
    <p class={css.meta} data-testid="dr-contact">{t('dr-contact')} {c().name} <Show when={c().email}><a href={`mailto:${c().email}`}>{c().email}</a></Show> <Show when={c().url}><a href={c().url ?? ''} rel="noopener noreferrer">{c().url}</a></Show></p>
  </Show>;
}

function RequestRow(props: { request: DataRequest; onChange: () => void }): JSX.Element {
  const [code, setCode] = createSignal('');
  const [message, setMessage] = createSignal('');
  const [trail, setTrail] = createSignal<TrailEntry[] | null>(null);
  const [busy, setBusy] = createSignal(false);
  const r = (): DataRequest => props.request;
  async function run(work: () => Promise<unknown>): Promise<void> {
    setBusy(true); setMessage('');
    try { await work(); props.onChange(); } catch (e) { setMessage(e instanceof Error ? e.message : t('dr-error')); } finally { setBusy(false); }
  }
  async function download(): Promise<void> {
    await run(async () => {
      const link = await downloadLink(r().id);
      const a = document.createElement('a');
      a.href = link.url; a.rel = 'noopener'; document.body.appendChild(a); a.click(); a.remove();
    });
  }
  async function history(): Promise<void> {
    if (trail()) { setTrail(null); return; }
    await run(async () => { setTrail((await loadRequest(r().id)).trail); });
  }
  return <li class={css.item} data-testid={`dr-${r().id}`}>
    <div class={css.itemMain}>
      <span class={css.itemName}>{t(`dr-kind-${r().kind}`)} <span class={css.badge} data-testid="dr-status">{t(`dr-status-${r().status}`)}</span></span>
      <span class={css.meta}>{t('dr-filed', { date: when(r().createdAt) })} · {t('dr-due', { date: when(r().dueAt) })}<Show when={r().extensionDays > 0}> · {t('dr-extended', { days: r().extensionDays })}</Show></span>
      <Show when={r().status === 'scheduled' && r().graceUntil}><span class={css.meta} data-testid="dr-grace">{t('dr-grace', { date: when(r().graceUntil ?? 0) })}</span></Show>
      <Show when={r().status === 'blocked'}><span class={css.meta}>{t('dr-blocked')}</span></Show>
      <Show when={r().status === 'rejected' && r().decisionReason}><span class={css.meta}>{t('dr-rejected-reason', { reason: r().decisionReason ?? '' })}</span></Show>
      <Show when={r().export}>{e => <span class={css.meta}>{t('dr-export-info', { size: Math.ceil(e().bytes / 1024), date: when(e().expiresAt) })}<Show when={!e().complete}> {t('dr-export-partial')}</Show></span>}</Show>
      <Show when={r().completion?.digest}><span class={css.meta}>{t('dr-record', { id: (r().completion?.digest ?? '').slice(0, 12) })}</span></Show>
      <Show when={r().needsConfirmation}>
        <div class={css.row}>
          <label class={css.label} for={`dr-code-${r().id}`}>{t('dr-code')}</label>
          <input id={`dr-code-${r().id}`} class={css.input} value={code()} autocomplete="one-time-code" onInput={e => setCode(e.currentTarget.value)} />
          <button type="button" class={css.button} disabled={busy() || code().trim() === ''} onClick={() => void run(() => confirmRequest(r().id, code()))}>{t('dr-confirm')}</button>
        </div>
        <span class={css.meta}>{t('dr-code-help')}</span>
      </Show>
      <Show when={message()}><p role="alert">{message()}</p></Show>
      <Show when={trail()}>{entries => <ol class={css.meta} aria-label={t('dr-history')}><For each={entries()}>{x => <li>{when(x.ts)} · {t(`dr-event-${x.kind}`)} ({t(`dr-by-${x.by}`)})</li>}</For></ol>}</Show>
    </div>
    <div class={css.actions}>
      <Show when={r().downloadAvailable}><button type="button" class={css.button} disabled={busy()} onClick={() => void download()}>{t('dr-download')}</button></Show>
      <Show when={r().canCancel}><button type="button" class={css.ghost} disabled={busy()} onClick={() => void run(() => cancelRequest(r().id))}>{r().status === 'scheduled' ? t('dr-cancel-erasure') : t('dr-cancel')}</button></Show>
      <button type="button" class={css.ghost} disabled={busy()} onClick={() => void history()}>{t('dr-history')}</button>
    </div>
  </li>;
}

export function DataRequests(): JSX.Element {
  onMount(() => { void loadCatalog('data-requests'); });
  const [overview] = createResource(async () => { try { return (await loadOverview()).policy; } catch { return null; } });
  const [requests, { refetch }] = createResource(async () => { try { return await listRequests(); } catch { return [] as DataRequest[]; } });
  const [kind, setKind] = createSignal<RequestKind>('access');
  // The Your data shortcuts (download, delete) pre-select a request type here.
  createEffect(() => { const k = requestedKind(); if (k) setKind(k); });
  const [details, setDetails] = createSignal('');
  const [includeMail, setIncludeMail] = createSignal(false);
  const [sure, setSure] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [notice, setNotice] = createSignal('');
  const offered = (p: DataRequestPolicy): RequestKind[] => REQUEST_KINDS.filter(k => p.types[k]);
  async function submit(p: DataRequestPolicy): Promise<void> {
    setBusy(true); setError(''); setNotice('');
    try {
      const k = offered(p).includes(kind()) ? kind() : offered(p)[0];
      if (!k) return;
      const created = await submitRequest({ kind: k, details: details(), includeMail: includeMail() && EXPORT_KINDS.includes(k), locale: locale() });
      setNotice(created.confirmationSent ? t('dr-submitted-code') : t('dr-submitted'));
      setDetails(''); setSure(false); setIncludeMail(false);
      await refetch();
    } catch (e) { setError(e instanceof Error ? e.message : t('dr-error')); } finally { setBusy(false); }
  }
  return <section id="your-data-requests" class={css.section} aria-labelledby="data-requests-heading" data-testid="data-requests">
    <h3 id="data-requests-heading" class={css.heading}>{t('dr-title')}</h3>
    <Show when={overview()} fallback={<p class={css.meta}>{overview.loading ? t('dr-loading') : t('dr-unavailable')}</p>}>{policy => <>
      <p class={css.prose}>{introText(policy(), locale())}</p>
      <Contact policy={policy()} />
      <Show when={policy().enabled && offered(policy()).length > 0} fallback={<p class={css.meta}>{t('dr-off')}</p>}>
        <div class={css.field}>
          <label class={css.label} for="dr-kind">{t('dr-kind')}</label>
          <select id="dr-kind" class={css.select} value={kind()} onChange={e => { setKind(e.currentTarget.value as RequestKind); setSure(false); }}>
            <For each={offered(policy())}>{k => <option value={k}>{t(`dr-kind-${k}`)}</option>}</For>
          </select>
          <p class={css.meta}>{t(`dr-kind-${kind()}-help`)}</p>
          <Show when={kind() === 'erasure'}><p class={css.meta} role="note">{t('dr-erasure-warning', { days: policy().graceDays })}</p></Show>
          <Show when={kind() === 'rectification'}><p class={css.meta}>{t('dr-rectification-pointer')}</p></Show>
          <Show when={EXPORT_KINDS.includes(kind()) && policy().mailAllowed}>
            <label><input type="checkbox" checked={includeMail()} onChange={e => setIncludeMail(e.currentTarget.checked)} /> {t('dr-include-mail')}</label>
          </Show>
          <label class={css.label} for="dr-details">{t('dr-details')}</label>
          <textarea id="dr-details" class={css.textarea} maxlength={2000} value={details()} onInput={e => setDetails(e.currentTarget.value)} />
          <Show when={policy().reauthRequired && ['access', 'portability', 'erasure'].includes(kind())}><p class={css.meta}>{t('dr-reauth')}</p></Show>
          <Show when={kind() === 'erasure'}>
            <label><input type="checkbox" checked={sure()} onChange={e => setSure(e.currentTarget.checked)} /> {t('dr-erasure-sure')}</label>
          </Show>
          <div class={css.actions}>
            <button type="button" class={kind() === 'erasure' ? css.danger : css.button} disabled={busy() || (kind() === 'erasure' && !sure())} onClick={() => void submit(policy())}>{t('dr-submit')}</button>
          </div>
          <Show when={error()}><p role="alert">{error()}</p></Show>
          <Show when={notice()}><p role="status">{notice()}</p></Show>
        </div>
      </Show>
    </>}</Show>
    <h4 class={css.subheading}>{t('dr-list')}</h4>
    <Show when={(requests() ?? []).length > 0} fallback={<p class={css.meta}>{t('dr-none')}</p>}>
      <ul class={css.list}><For each={requests() ?? []}>{r => <RequestRow request={r} onChange={() => void refetch()} />}</For></ul>
    </Show>
  </section>;
}
