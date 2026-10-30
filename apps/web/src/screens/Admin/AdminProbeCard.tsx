// Security centre › Adaptive protection › Admin probe protection (t97 on t100's
// server). Explains the policy in plain words, edits each setting in place
// (bans.write + step-up, If-Match on the policy ETag), lists the current blocks
// with Unblock, and shows the counters and the top offenders.
//
// A block refuses every request from that source, the mail app included, so the
// card suggests allowlisting shared company networks.

import { createSignal, For, onMount, Show, useContext, type JSX } from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';
import { adminProbeApi, type AdminProbeApi, type AdminProbeKey, type AdminProbeOverview, type AdminProbePolicy, type AdminProbeSignal, type AdminProbeSource } from '../../api/admin/adminProbe.ts';
import { t, formatNumber, formatDateTime } from '../../i18n';
import { formatDurationMs } from '../../i18n/format.ts';
import { ShellContext } from './shell/context.ts';
import {useSavingToast} from './shell/useSavingToast.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { sameValue } from './protectionRules.ts';
import * as css from './protectionPolicy.css.ts';
import * as loading from '../../components/asyncState.css.ts';

type Kind = { type: 'bool' } | { type: 'int'; min: number; max: number; unit?: 's' | 'ms' } | { type: 'enum'; values: readonly string[] } | { type: 'signals' } | { type: 'list'; max: number };
/** The 14 settings, with the bounds and defaults of the server registry. */
export const PROBE_FIELDS: readonly { key: AdminProbeKey; kind: Kind; def: unknown }[] = [
  { key: 'enabled', kind: { type: 'bool' }, def: true },
  { key: 'graceAllowance', kind: { type: 'int', min: 0, max: 1000 }, def: 10 },
  { key: 'windowSecs', kind: { type: 'int', min: 60, max: 86_400, unit: 's' }, def: 600 },
  { key: 'blockAfter', kind: { type: 'int', min: 1, max: 1000 }, def: 10 },
  { key: 'tarpitDelayMs', kind: { type: 'int', min: 0, max: 10_000, unit: 'ms' }, def: 1500 },
  { key: 'blockSecs', kind: { type: 'int', min: 60, max: 604_800, unit: 's' }, def: 900 },
  { key: 'maxBlockSecs', kind: { type: 'int', min: 60, max: 2_592_000, unit: 's' }, def: 86_400 },
  { key: 'repeatWindowSecs', kind: { type: 'int', min: 0, max: 2_592_000, unit: 's' }, def: 86_400 },
  { key: 'blockStatus', kind: { type: 'enum', values: ['429', '403'] }, def: '429' },
  { key: 'signals', kind: { type: 'signals' }, def: null },
  { key: 'probePaths', kind: { type: 'list', max: 100 }, def: null },
  { key: 'allowlist', kind: { type: 'list', max: 100 }, def: [] },
  { key: 'auditIp', kind: { type: 'enum', values: ['masked', 'full'] }, def: 'masked' },
  { key: 'fail2banLog', kind: { type: 'bool' }, def: true },
];
export const PROBE_SIGNALS: readonly AdminProbeSignal[] = ['probe-path', 'slug-guess', 'admin-signin-failed', 'admin-unauthenticated'];
const slug = (key: string): string => key.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);

export function probeDuration(value: number, unit: 's' | 'ms'): string {
  return formatDurationMs(unit === 's' ? value * 1000 : value);
}
/** A client check mirroring the server's; the server decides. */
export function validateProbe(key: AdminProbeKey, value: unknown, policy: AdminProbePolicy): string | null {
  const field = PROBE_FIELDS.find((f) => f.key === key);
  if (!field) return null;
  const k = field.kind;
  if (k.type === 'int') {
    if (typeof value !== 'number' || !Number.isInteger(value)) return t('admin-probe-invalid-whole');
    if (value < k.min) return t('admin-probe-invalid-min', { min: k.min });
    if (value > k.max) return t('admin-probe-invalid-max', { max: k.max });
    if (key === 'blockSecs' && value > policy.maxBlockSecs) return t('admin-probe-invalid-block-order');
    if (key === 'maxBlockSecs' && value < policy.blockSecs) return t('admin-probe-invalid-block-order');
  }
  if (k.type === 'list' && Array.isArray(value)) {
    if (value.length > k.max) return t('admin-probe-invalid-items', { items: k.max });
    if (key === 'probePaths' && value.some((p) => typeof p !== 'string' || !p.startsWith('/'))) return t('admin-probe-invalid-path');
  }
  return null;
}

export function AdminProbeCard(props: { api?: AdminProbeApi | undefined }): JSX.Element {
  const shell = useContext(ShellContext);
  const api = (): AdminProbeApi => props.api ?? adminProbeApi;
  const canWrite = (): boolean => !shell || shell.can.can('bans.write');
  const [state, setState] = createStore<{ data: AdminProbeOverview | null; etag: string | null }>({ data: null, etag: null });
  const [blocks, setBlocks] = createSignal<AdminProbeSource[]>([]);
  const [error, setError] = createSignal('');
  const [drafts, setDrafts] = createStore<Partial<Record<AdminProbeKey, unknown>>>({});
  const [rowError, setRowError] = createStore<Partial<Record<AdminProbeKey, string>>>({});
  const [busy, setBusy] = createSignal<string | null>(null);
  useSavingToast(()=>busy()!==null,()=>t('admin-protect-saving'));
  const [saved, setSaved] = createSignal('');
  const [unblocking, setUnblocking] = createSignal<AdminProbeSource>();

  async function load(): Promise<void> {
    try {
      const [overview, list] = await Promise.all([api().overview(), api().blocks()]);
      if (state.data) setState('data', reconcile(overview.data)); else setState('data', overview.data);
      setState('etag', overview.etag);
      setBlocks(list);
      setError('');
    } catch (cause) {
      setError(describeAdminError(cause));
    }
  }
  onMount(() => void load());

  const policy = (): AdminProbePolicy | undefined => state.data?.policy;
  async function save(key: AdminProbeKey, value: unknown): Promise<void> {
    const p = policy();
    if (!p) return;
    const problem = value === null ? null : validateProbe(key, value, p);
    if (problem) { setRowError(key, problem); return; }
    setBusy(key);
    setSaved('');
    try {
      const result = await api().save({ [key]: value }, state.etag);
      setState('data', reconcile(result.data));
      setState('etag', result.etag);
      setDrafts(key, undefined);
      setRowError(key, '');
      setSaved(t('admin-probe-saved', { setting: t(`admin-settings-key-security-admin-probe-${slug(key)}`) }));
    } catch (cause) {
      setRowError(key, describeAdminError(cause));
    } finally {
      setBusy(null);
    }
  }
  async function unblock(row: AdminProbeSource): Promise<void> {
    await api().unblock(row.id);
    setUnblocking(undefined);
    setSaved(t('admin-probe-unblocked', { source: row.source }));
    await load();
  }
  const when = (iso: string | null): string => (iso ? formatDateTime(iso) : '—');
  return (
    <section class={css.card} aria-labelledby="admin-probe-h" data-testid="admin-probe-card">
      <h3 id="admin-probe-h" class={css.title}>{t('admin-probe-title')}</h3>
      <p class={css.help}>{t('admin-probe-intro')}</p>
      <p class={css.warnNote} data-testid="admin-probe-mail-warning">{t('admin-probe-mail-app-warning')}</p>
      <Feedback kind="success" as="toast" message={saved()} />
      <Show when={error()}><Feedback kind="error" message={error()} /></Show>
      <Show when={state.data} fallback={<Show when={!error()}><div class={loading.skeletonCards} aria-hidden="true"><span class={loading.skeletonCard} /></div></Show>}>
        {(d) => <>
          <ProbeSummary policy={d().policy} />
          <div class={css.counters} data-testid="admin-probe-counters">
            <Counter label={t('admin-probe-stat-active')} value={d().stats.activeBlocks} />
            <Counter label={t('admin-probe-stat-24h')} value={d().stats.blocksLast24h} />
            <Counter label={t('admin-probe-stat-total')} value={d().stats.blocksTotal} />
            <Counter label={t('admin-probe-stat-refused')} value={d().stats.refusedRequests} />
            <Counter label={t('admin-probe-stat-tarpitted')} value={d().stats.tarpittedRequests} />
            <Counter label={t('admin-probe-stat-cleared')} value={d().stats.clearedBySignin} />
          </div>
          <p class={css.note}>{t('admin-probe-stat-scope', { since: when(d().stats.since), tracked: d().stats.trackedSources, capacity: d().stats.capacity, seconds: d().propagationSecs })}</p>

          <h4 class={css.title}>{t('admin-probe-blocks-title')}</h4>
          <Show when={blocks().length > 0} fallback={<p class={css.note} data-testid="admin-probe-no-blocks">{t('admin-probe-blocks-empty')}</p>}>
            <table class={css.diff} data-testid="admin-probe-blocks">
              <caption class={css.srCaption}>{t('admin-probe-blocks-title')}</caption>
              <thead><tr><th scope="col">{t('admin-probe-col-source')}</th><th scope="col">{t('admin-probe-col-until')}</th><th scope="col">{t('admin-probe-col-strikes')}</th><th scope="col">{t('admin-probe-col-last')}</th><th scope="col"><span class={css.srCaption}>{t('admin-probe-col-actions')}</span></th></tr></thead>
              <tbody><For each={blocks()}>{(row) => (
                <tr data-source={row.source}>
                  <th scope="row"><code>{row.source}</code></th>
                  <td>{when(row.blockedUntil)}</td>
                  <td>{formatNumber(row.strikes)}</td>
                  <td>{row.lastSignal ? t(`admin-probe-signal-${row.lastSignal}`) : '—'}</td>
                  <td><Show when={canWrite()}><button class="btn btn--ghost btn--sm" aria-label={t('admin-probe-unblock-target', { source: row.source })} onClick={() => setUnblocking(row)}>{t('admin-probe-unblock')}</button></Show></td>
                </tr>
              )}</For></tbody>
            </table>
          </Show>

          <h4 class={css.title}>{t('admin-probe-offenders-title')}</h4>
          <Show when={d().topOffenders.length > 0} fallback={<p class={css.note}>{t('admin-probe-offenders-empty')}</p>}>
            <table class={css.diff} data-testid="admin-probe-offenders">
              <caption class={css.srCaption}>{t('admin-probe-offenders-title')}</caption>
              <thead><tr><th scope="col">{t('admin-probe-col-source')}</th><th scope="col">{t('admin-probe-col-score')}</th><th scope="col">{t('admin-probe-col-misses')}</th><th scope="col">{t('admin-probe-col-state')}</th></tr></thead>
              <tbody><For each={d().topOffenders}>{(row) => (
                <tr data-fired={row.blocked ? 'true' : 'false'}>
                  <th scope="row"><code>{row.source}</code></th>
                  <td>{t('admin-probe-score', { score: row.score, grace: row.graceAllowance })}</td>
                  <td>{formatNumber(row.signals)}</td>
                  <td>{row.blocked ? t('admin-probe-state-blocked') : t('admin-probe-state-watched', { last: when(row.lastSeen) })}</td>
                </tr>
              )}</For></tbody>
            </table>
          </Show>

          <h4 class={css.title}>{t('admin-probe-settings-title')}</h4>
          <Show when={!canWrite()}><p class={css.note} data-testid="admin-probe-readonly">{t('admin-probe-readonly')}</p></Show>
          <ul class={css.rules}>
            <For each={PROBE_FIELDS}>{(field) => (
              <ProbeRow field={field} value={d().policy[field.key]} draft={drafts[field.key]} error={rowError[field.key] ?? ''}
                busy={busy() === field.key} disabled={!canWrite() || (busy() !== null && busy() !== field.key)}
                onDraft={(v) => { setDrafts(field.key, v); setRowError(field.key, ''); }}
                onSave={(v) => void save(field.key, v)} onReset={() => void save(field.key, null)} />
            )}</For>
          </ul>
        </>}
      </Show>
      <ConfirmDialog open={unblocking() !== undefined} title={t('admin-probe-unblock-target', { source: unblocking()?.source ?? '' })} body={t('admin-probe-unblock-body')}
        confirmLabel={t('admin-probe-unblock')} onCancel={() => setUnblocking(undefined)} onConfirm={async () => { const row = unblocking(); if (row) await unblock(row); }} />
    </section>
  );
}

function Counter(props: { label: string; value: number }): JSX.Element {
  return <div class={css.counter}><span class={css.note}>{props.label}</span><strong>{formatNumber(props.value)}</strong></div>;
}

function ProbeSummary(props: { policy: AdminProbePolicy }): JSX.Element {
  const p = (): AdminProbePolicy => props.policy;
  return <ul class={css.summary} data-testid="admin-probe-summary">
    <li><strong>{t(p().enabled ? 'admin-probe-summary-on' : 'admin-probe-summary-off')}</strong></li>
    <Show when={p().enabled}>
      <li>{t('admin-probe-summary-grace', { grace: p().graceAllowance, window: probeDuration(p().windowSecs, 's') })}</li>
      <li>{t('admin-probe-summary-tarpit', { after: p().blockAfter, delay: probeDuration(p().tarpitDelayMs, 'ms') })}</li>
      <li>{t('admin-probe-summary-block', { first: probeDuration(p().blockSecs, 's'), max: probeDuration(p().maxBlockSecs, 's'), repeat: probeDuration(p().repeatWindowSecs, 's'), status: p().blockStatus })}</li>
      <li>{t('admin-probe-summary-allowlist', { count: p().allowlist.length })}</li>
    </Show>
  </ul>;
}

function ProbeRow(props: {
  field: (typeof PROBE_FIELDS)[number]; value: unknown; draft: unknown; error: string; busy: boolean; disabled: boolean;
  onDraft: (v: unknown) => void; onSave: (v: unknown) => void; onReset: () => void;
}): JSX.Element {
  const key = (): AdminProbeKey => props.field.key;
  const id = (): string => `admin-probe-${slug(key())}`;
  const label = (): string => t(`admin-settings-key-security-admin-probe-${slug(key())}`);
  const shown = (): unknown => (props.draft !== undefined ? props.draft : props.value);
  const changed = (): boolean => props.draft !== undefined && !sameValue(props.draft, props.value);
  const isDefault = (): boolean => props.field.def !== null && sameValue(props.value, props.field.def);
  const control = (): JSX.Element => {
    const k = props.field.kind;
    if (k.type === 'bool') return <input id={id()} type="checkbox" checked={shown() === true} disabled={props.disabled} onChange={(e) => props.onDraft(e.currentTarget.checked)} />;
    if (k.type === 'enum') return <select id={id()} value={String(shown())} disabled={props.disabled} onChange={(e) => props.onDraft(e.currentTarget.value)}>
      <For each={k.values}>{(v) => <option value={v}>{t(`admin-probe-value-${v}`)}</option>}</For>
    </select>;
    if (k.type === 'signals') {
      const list = (): string[] => (shown() as string[] | undefined) ?? [];
      return <fieldset id={id()} class={css.signals} disabled={props.disabled}><legend class={css.srLegend}>{label()}</legend>
        <For each={PROBE_SIGNALS}>{(s) => <label class={css.signal}><input type="checkbox" checked={list().includes(s)} onChange={(e) => props.onDraft(e.currentTarget.checked ? [...list(), s] : list().filter((x) => x !== s))} />{t(`admin-probe-signal-${s}`)}</label>}</For>
      </fieldset>;
    }
    if (k.type === 'list') return <textarea id={id()} rows={key() === 'probePaths' ? 6 : 3} disabled={props.disabled} value={((shown() as string[] | undefined) ?? []).join('\n')}
      onInput={(e) => props.onDraft(e.currentTarget.value.split(/[\n,]/).map((x) => x.trim()).filter((x) => x !== ''))} />;
    return <span class={css.number}>
      <input id={id()} type="number" inputmode="numeric" min={k.min} max={k.max} step="1" value={String(shown() ?? '')} disabled={props.disabled}
        onInput={(e) => { const n = e.currentTarget.valueAsNumber; props.onDraft(Number.isNaN(n) ? e.currentTarget.value : n); }} />
      <Show when={k.unit && typeof shown() === 'number'}><span class={css.note}>{probeDuration(shown() as number, k.unit!)}</span></Show>
    </span>;
  };
  return (
    <li class={css.rule} data-testid={`admin-probe-row-${slug(key())}`} data-changed={changed() ? 'true' : 'false'}>
      <div class={css.ruleHead}><label for={id()} class={css.ruleLabel}>{label()}</label><code class={css.key}>security.adminProbe.{key()}</code></div>
      <p class={css.help}>{t(`admin-settings-help-security-admin-probe-${slug(key())}`)}</p>
      <Show when={key() === 'allowlist'}><p class={css.example}><span class={css.exampleTag}>{t('admin-protect-example')}</span> {t('admin-probe-allowlist-example')}</p></Show>
      <div class={css.ruleControls}>{control()}
        <Show when={props.field.def !== null}><span class={css.note}>{isDefault() ? t('admin-protect-is-default') : t('admin-probe-default', { value: describeProbe(props.field, props.field.def) })}</span></Show>
      </div>
      <Show when={props.error}><p class={css.error} role="alert">{props.error}</p></Show>
      <div class={css.actions}>
        <button class="btn btn--sm" disabled={props.disabled || !changed() || props.busy} onClick={() => props.onSave(props.draft)}
          aria-busy={props.busy} aria-label={t('admin-protect-save-rule', { rule: label() })}>{t('admin-protect-save')}</button>
        <Show when={changed()}><button class="btn btn--ghost btn--sm" onClick={() => props.onDraft(undefined)}>{t('admin-protect-cancel')}</button></Show>
        <button class="btn btn--ghost btn--sm" disabled={props.disabled || isDefault() || props.busy} onClick={() => props.onReset()}
          aria-label={t('admin-protect-reset-rule', { rule: label() })}>{t('admin-protect-reset')}</button>
      </div>
    </li>
  );
}

function describeProbe(field: (typeof PROBE_FIELDS)[number], v: unknown): string {
  const k = field.kind;
  if (typeof v === 'boolean') return t(v ? 'admin-protect-on' : 'admin-protect-off');
  if (k.type === 'enum') return t(`admin-probe-value-${String(v)}`);
  if (Array.isArray(v)) return v.length === 0 ? t('admin-protect-none') : v.join(', ');
  if (typeof v === 'number') return k.type === 'int' && k.unit ? probeDuration(v, k.unit) : formatNumber(v);
  return String(v ?? '');
}
