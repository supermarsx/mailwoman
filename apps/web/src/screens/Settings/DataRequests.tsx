// Settings > Privacy & data > Your data: file and follow data requests (access,
// download, correction, erasure, restriction, objection, other). What is offered,
// the approval steps and the grace period are the administrator's `dsr.*` settings.
import { createEffect, createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { activeLocaleSignal as locale } from '../../i18n/registry.ts';
import { formatDateTime } from '../../i18n/format.ts';
import {
  OBJECTION_PURPOSES, REQUEST_KINDS, cancelRequest, confirmRequest, downloadLink, listRequests, loadOverview, loadRequest, requestReference, submitRequest,
  type DataRequest, type DataRequestPolicy, type RequestKind, type TrailEntry,
} from '../../api/dataRequests.ts';
import { requestedKind } from '../../api/privacyShortcut.ts';
import { ControlRow } from './layout/SettingRow.tsx';
import { useSettingsNavigation } from './navigation.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
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
  const navigation = useSettingsNavigation();
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
      <Show when={r().kind === 'objection' && (r().purposes?.length ?? 0) > 0}><span class={css.meta} data-testid="dr-purposes">{t('dr-objection-purposes-list', { purposes: (r().purposes ?? []).map(p => t(`dr-purpose-${p.split('.')[0]}`)).join(', ') })}</span></Show>
      <Show when={r().kind === 'rectification' && navigation && ['awaiting_approval', 'in_progress', 'queued', 'awaiting_confirmation', 'awaiting_verification'].includes(r().status)}>
        <span class={css.meta} data-testid="dr-rectify">
          {t('dr-rectify-ref', { ref: requestReference(r().id) })}{' '}
          <button type="button" class={css.ghost} onClick={() => void navigation?.navigate({ section: 'identities', rectification: { requestId: r().id, reference: requestReference(r().id) } })}>{t('dr-rectify-edit')}</button>
        </span>
      </Show>
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
  const [objectTo, setObjectTo] = createSignal<readonly string[]>([]);
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
      const created = await submitRequest({ kind: k, details: details(), ...(k === 'objection' && objectTo().length > 0 ? { purposes: [...objectTo()] } : {}), includeMail: includeMail() && EXPORT_KINDS.includes(k), locale: locale() });
      setNotice(created.confirmationSent ? t('dr-submitted-code') : t('dr-submitted'));
      setDetails(''); setSure(false); setIncludeMail(false); setObjectTo([]);
      await refetch();
    } catch (e) { setError(e instanceof Error ? e.message : t('dr-error')); } finally { setBusy(false); }
  }
  return <>
  <SettingsCard id="your-data-requests" wide title={t('dr-title')} testId="data-requests"
    {...(overview() ? { description: introText(overview()!, locale()) } : {})}>
    <Show when={overview()} fallback={<p class={css.meta}>{overview.loading ? t('dr-loading') : t('dr-unavailable')}</p>}>{policy => <>
      <Contact policy={policy()} />
      <Show when={policy().enabled && offered(policy()).length > 0} fallback={<p class={css.meta}>{t('dr-off')}</p>}>
        <ControlRow controlId="dr-kind" label={t('dr-kind')} help={t(`dr-kind-${kind()}-help`)}
          note={<>
            <Show when={kind() === 'erasure'}><p class={css.meta} role="note">{t('dr-erasure-warning', { days: policy().graceDays })}</p></Show>
            <Show when={kind() === 'rectification'}><p class={css.meta}>{t('dr-rectification-pointer')}</p></Show>
          </>}>{() =>
          <select id="dr-kind" class={css.select} value={kind()} onChange={e => { setKind(e.currentTarget.value as RequestKind); setSure(false); }}>
            <For each={offered(policy())}>{k => <option value={k}>{t(`dr-kind-${k}`)}</option>}</For>
          </select>
        }</ControlRow>
        <Show when={EXPORT_KINDS.includes(kind()) && policy().mailAllowed}>
          <ControlRow controlId="dr-include-mail" label={t('dr-include-mail')}>{() =>
            <input id="dr-include-mail" type="checkbox" checked={includeMail()} onChange={e => setIncludeMail(e.currentTarget.checked)} />
          }</ControlRow>
        </Show>
        <Show when={kind() === 'objection'}>
          <fieldset class={css.bareFieldset} data-testid="dr-objection-purposes">
            <legend class={css.label}>{t('dr-objection-purposes')}</legend>
            <p class={css.meta}>{t('dr-objection-purposes-help')}</p>
            <For each={OBJECTION_PURPOSES}>{p => <label class={css.check}><input type="checkbox" class={css.checkbox} checked={objectTo().includes(p)} onChange={e => setObjectTo(cur => e.currentTarget.checked ? [...cur, p] : cur.filter(x => x !== p))} />{t(`dr-purpose-${p}`)}</label>}</For>
          </fieldset>
        </Show>
        <ControlRow controlId="dr-details" stacked label={t('dr-details')}>{() =>
          <textarea id="dr-details" class={css.textarea} maxlength={2000} value={details()} onInput={e => setDetails(e.currentTarget.value)} />
        }</ControlRow>
        <Show when={policy().reauthRequired && ['access', 'portability', 'erasure'].includes(kind())}><p class={css.meta}>{t('dr-reauth')}</p></Show>
        <Show when={kind() === 'erasure'}>
          <ControlRow controlId="dr-erasure-sure" label={t('dr-erasure-sure')}>{() =>
            <input id="dr-erasure-sure" type="checkbox" checked={sure()} onChange={e => setSure(e.currentTarget.checked)} />
          }</ControlRow>
        </Show>
        <div class={css.actions}>
          <button type="button" class={kind() === 'erasure' ? css.danger : css.button} disabled={busy() || (kind() === 'erasure' && !sure())} onClick={() => void submit(policy())}>{t('dr-submit')}</button>
        </div>
        <Show when={error()}><p role="alert">{error()}</p></Show>
        <Show when={notice()}><p role="status">{notice()}</p></Show>
      </Show>
    </>}</Show>
  </SettingsCard>
  <SettingsCard id="data-requests-card-list" wide title={t('dr-list')}>
    <Show when={(requests() ?? []).length > 0} fallback={<p class={css.meta}>{t('dr-none')}</p>}>
      <ul class={css.list}><For each={requests() ?? []}>{r => <RequestRow request={r} onChange={() => void refetch()} />}</For></ul>
    </Show>
  </SettingsCard>
  </>;
}
