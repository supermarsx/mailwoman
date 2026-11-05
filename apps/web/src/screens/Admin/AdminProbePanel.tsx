// Request protection › Admin probe subtab: every admin-probe policy in groups,
// live counters beside them (t97 on t100's
// server). Explains the policy in plain words, edits each setting in place
// (bans.write + step-up, If-Match on the policy ETag), lists the current blocks
// with Unblock, and shows the counters and the top offenders.
//
// A block refuses every request from that source, the mail app included, so the
// card suggests allowlisting shared company networks.

import { createResource, createSignal, For, onMount, Show, useContext, type JSX } from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';
import { adminProbeApi, type AdminProbeApi, type AdminProbeKey, type AdminProbeOverview, type AdminProbePolicy, type AdminProbeSignal, type AdminProbeSource, type AdminProbeStats } from '../../api/admin/adminProbe.ts';
import { t, formatNumber, formatDateTime, formatRelative, loadCatalog } from '../../i18n';
import { listAdminSettings, type AdminSetting } from '../../api/admin/settings.ts';
import { resourceValue } from '../../components/resourceValue.ts';
import { AdminSubsection } from './AdminSubsection.tsx';
import { adminDocsHref } from './shell/docsLink.ts';
import { InlineSettingEditor } from './Settings/InlineSettingEditor.tsx';
import { Icon } from './shell/icons.tsx';
import * as pg from './protocolGuard.css.ts';
import * as admin from './admin.css.ts';
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

/** The policy groups of the Admin probe subtab: dedicated-route keys (`probe`) and registry keys (`registry`). */
export const PROBE_GROUPS: readonly { id: string; probe: readonly AdminProbeKey[]; registry: readonly string[] }[] = [
  { id: 'detection', probe: ['signals', 'probePaths', 'graceAllowance', 'windowSecs', 'blockAfter'], registry: [] },
  { id: 'responses', probe: ['tarpitDelayMs', 'blockSecs', 'maxBlockSecs', 'repeatWindowSecs', 'blockStatus'], registry: [] },
  {
    id: 'countries', probe: [],
    registry: ['adminGeoEnabled', 'adminGeoLookupFailure', 'adminGeoThreshold', 'adminGeoWindowSecs', 'adminGeoBlockOnFirst', 'adminGeoBlockSecs', 'adminGeoSecondBlockSecs', 'adminGeoMaxBlockSecs', 'adminGeoRepeatDays'].map((k) => `security.trust.${k}`),
  },
  { id: 'exemptions', probe: ['allowlist'], registry: ['security.trust.adminGeoCountries'] },
  { id: 'logging', probe: ['auditIp', 'fail2banLog'], registry: [] },
];
/** Every dedicated-route key is shown once, `enabled` through the mode switch. */
export const PROBE_GROUP_KEYS: readonly AdminProbeKey[] = ['enabled', ...PROBE_GROUPS.flatMap((g) => g.probe)];

/** Per-evidence counts with the latest time a source last showed that evidence (from the top offenders). */
export function evidenceRows(stats: AdminProbeStats, offenders: readonly AdminProbeSource[]): { signal: AdminProbeSignal; count: number; lastSeen: string | null }[] {
  return PROBE_SIGNALS.map((signal) => {
    const seen = offenders.filter((o) => o.lastSignal === signal && o.lastSeen).map((o) => o.lastSeen as string).sort();
    return { signal, count: stats.signals[signal] ?? 0, lastSeen: seen.length > 0 ? seen[seen.length - 1]! : null };
  }).sort((a, b) => b.count - a.count || PROBE_SIGNALS.indexOf(a.signal) - PROBE_SIGNALS.indexOf(b.signal));
}

export function AdminProbePanel(props: { api?: AdminProbeApi | undefined; load?: typeof listAdminSettings | undefined }): JSX.Element {
  void loadCatalog('admin-settings');
  const shell = useContext(ShellContext);
  const api = (): AdminProbeApi => props.api ?? adminProbeApi;
  const canWrite = (): boolean => !shell || shell.can.can('bans.write');
  const canReadSettings = (): boolean => !shell || shell.can.can('settings.read');
  const [state, setState] = createStore<{ data: AdminProbeOverview | null; etag: string | null }>({ data: null, etag: null });
  const [blocks, setBlocks] = createSignal<AdminProbeSource[]>([]);
  const [error, setError] = createSignal('');
  const [drafts, setDrafts] = createStore<Partial<Record<AdminProbeKey, unknown>>>({});
  const [rowError, setRowError] = createStore<Partial<Record<AdminProbeKey, string>>>({});
  const [busy, setBusy] = createSignal<string | null>(null);
  useSavingToast(() => busy() !== null, () => t('admin-protect-saving'));
  const [saved, setSaved] = createSignal('');
  const [unblocking, setUnblocking] = createSignal<AdminProbeSource>();
  const [confirmOn, setConfirmOn] = createSignal(false);
  const [showZero, setShowZero] = createSignal(true);
  const [settings, { mutate }] = createResource(canReadSettings, async () => await (props.load ?? listAdminSettings)());
  const settingsNow = () => resourceValue(settings);
  const rowOf = (key: string): AdminSetting | undefined => {
    const r = settingsNow();
    return r?.kind === 'ok' ? r.data.find((s) => s.key === key) : undefined;
  };
  const onSaved = (row: AdminSetting): void => {
    mutate((value) => (value?.kind === 'ok' ? { ...value, data: value.data.map((c) => (c.key === row.key ? row : c)) } : value));
    void load();
  };

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
  const reason = (signal: AdminProbeSignal | null | undefined): string => signal && PROBE_SIGNALS.includes(signal) ? t(`admin-probe-signal-${signal}`) : '—';
  const chooseMode = (on: boolean): void => {
    if (on === (policy()?.enabled ?? false)) return;
    if (on) setConfirmOn(true); else void save('enabled', false);
  };
  const lastSeenOverall = (d: AdminProbeOverview): string | null => {
    const all = d.topOffenders.filter((o) => o.lastSeen).map((o) => o.lastSeen as string).sort();
    return all.length > 0 ? all[all.length - 1]! : null;
  };
  const rel = (iso: string | null): JSX.Element => iso
    ? <time datetime={iso} title={formatDateTime(iso)}>{formatRelative(iso)}</time>
    : <span class={pg.zero}>{t('admin-protocol-never')}</span>;
  const tile = (value: string, label: string): JSX.Element => <div class={pg.tile}><span class={pg.tileValue}>{value}</span><span class={pg.tileLabel}>{label}</span></div>;
  const probeRow = (key: AdminProbeKey, d: AdminProbeOverview): JSX.Element => {
    const field = PROBE_FIELDS.find((f) => f.key === key)!;
    return <ProbeRow field={field} value={d.policy[key]} draft={drafts[key]} error={rowError[key] ?? ''}
      busy={busy() === key} disabled={!canWrite() || (busy() !== null && busy() !== key)}
      onDraft={(v) => { setDrafts(key, v); setRowError(key, ''); }}
      onSave={(v) => void save(key, v)} onReset={() => void save(key, null)} />;
  };
  return (
    <section aria-label={t('admin-probe-title')} data-testid="admin-probe-panel">
      <div class={admin.card} data-testid="admin-probe-card">
        <h3 class={admin.heading}>{t('admin-probe-title')}</h3>
        <p class={admin.note}>{t('admin-probe-intro')} <a href={adminDocsHref('security-center')} target="_blank" rel="noopener noreferrer">{t('admin-ux-learn-more')}</a></p>
        <p class={css.warnNote} data-testid="admin-probe-mail-warning">{t('admin-probe-mail-app-warning')}</p>
        <Feedback kind="success" as="toast" message={saved()} />
        <Show when={error()}><Feedback kind="error" message={error()} /></Show>
        <Show when={!canWrite()}><p class={admin.note} data-testid="admin-probe-readonly">{t('admin-probe-readonly')}</p></Show>
        <Show when={state.data} fallback={<Show when={!error()}><div class={loading.skeletonCards} aria-hidden="true"><span class={loading.skeletonCard} /></div></Show>}>
          {(d) => <>
            <div class={pg.modeSwitch} role="group" aria-label={t('admin-probe-mode')}>
              <For each={[false, true]}>{(on) => (
                <button class="btn" type="button" aria-pressed={d().policy.enabled === on} disabled={!canWrite() || busy() !== null} onClick={() => chooseMode(on)} data-testid={`admin-probe-mode-${on ? 'on' : 'off'}`}>
                  {t(on ? 'admin-probe-mode-on' : 'admin-probe-mode-off')}
                </button>
              )}</For>
              <span>{t('admin-protocol-overall-now')} <strong>{t(d().policy.enabled ? 'admin-probe-mode-on' : 'admin-probe-mode-off')}</strong></span>
            </div>
            <p class={admin.note}>{t('admin-probe-mode-help')}</p>
            <ProbeSummary policy={d().policy} />
          </>}
        </Show>
      </div>
      <Show when={state.data}>
        {(d) => <>
          <div class={admin.card} data-testid="admin-probe-stats">
            <h3 class={admin.heading}>{t('admin-probe-stats-title')}</h3>
            <div class={pg.tiles} data-testid="admin-probe-counters">
              {tile(formatNumber(d().stats.activeBlocks), t('admin-probe-stat-active'))}
              {tile(formatNumber(d().stats.blocksLast24h), t('admin-probe-stat-24h'))}
              {tile(formatNumber(d().stats.blocksTotal), t('admin-probe-stat-total'))}
              {tile(formatNumber(d().stats.refusedRequests), t('admin-probe-stat-refused'))}
              {tile(formatNumber(d().stats.tarpittedRequests), t('admin-probe-stat-tarpitted'))}
              {tile(formatNumber(d().stats.clearedBySignin), t('admin-probe-stat-cleared'))}
            </div>
            <p class={admin.note}>{t('admin-probe-stat-scope', { since: when(d().stats.since), tracked: d().stats.trackedSources, capacity: d().stats.capacity, seconds: d().propagationSecs })}</p>
            <p class={admin.note}>{t('admin-probe-last-seen')} {rel(lastSeenOverall(d()))}</p>
            <Show when={d().trustTable}>{(tt) => <p class={admin.note} data-testid="admin-trust-table">{t('admin-probe-trust-table', { entries: tt().entries, restored: tt().restoredAtStartup, saved: tt().lastSaved ? formatDateTime(new Date(tt().lastSaved! * 1000).toISOString()) : '—', persist: t(tt().persist ? 'admin-probe-trust-table-on' : 'admin-probe-trust-table-off') })}</p>}</Show>
            <div class={pg.toolbar}>
              <h4 class={admin.heading}>{t('admin-probe-evidence-title')}</h4>
              <button class="btn btn--ghost" aria-pressed={!showZero()} onClick={() => setShowZero(!showZero())}>{showZero() ? t('admin-protocol-hide-zero') : t('admin-protocol-show-all')}</button>
            </div>
            <table class={pg.table} data-testid="admin-probe-evidence">
              <caption class={css.srCaption}>{t('admin-probe-evidence-title')}</caption>
              <thead><tr><th scope="col">{t('admin-probe-col-reason')}</th><th scope="col" class="num">{t('admin-probe-col-misses')}</th><th scope="col">{t('admin-probe-col-last')}</th></tr></thead>
              <tbody><For each={evidenceRows(d().stats, d().topOffenders).filter((r) => showZero() || r.count > 0)}>{(r) => (
                <tr data-signal={r.signal}>
                  <td class={pg.label}>{t(`admin-probe-signal-${r.signal}`)}</td>
                  <td class="num" data-label={t('admin-probe-col-misses')}>
                    <Show when={r.count > 0} fallback={<span class={pg.zero}>{formatNumber(0)}</span>}><span class={pg.hit}><Icon name="alert" />{formatNumber(r.count)}</span></Show>
                  </td>
                  <td data-label={t('admin-probe-col-last')}>{rel(r.lastSeen)}</td>
                </tr>
              )}</For></tbody>
            </table>

            <h4 class={admin.heading}>{t('admin-probe-blocks-title')}</h4>
            <Show when={blocks().length > 0} fallback={<p class={admin.note} data-testid="admin-probe-no-blocks">{t('admin-probe-blocks-empty')}</p>}>
              <table class={pg.table} data-testid="admin-probe-blocks">
                <caption class={css.srCaption}>{t('admin-probe-blocks-title')}</caption>
                <thead><tr><th scope="col">{t('admin-probe-col-source')}</th><th scope="col">{t('admin-probe-col-until')}</th><th scope="col" class="num">{t('admin-probe-col-strikes')}</th><th scope="col">{t('admin-probe-col-last')}</th><th scope="col"><span class={css.srCaption}>{t('admin-probe-col-actions')}</span></th></tr></thead>
                <tbody><For each={blocks()}>{(row) => (
                  <tr data-source={row.source}>
                    <th scope="row"><code>{row.source}</code></th>
                    <td data-label={t('admin-probe-col-until')}>{when(row.blockedUntil)}</td>
                    <td class="num" data-label={t('admin-probe-col-strikes')}>{formatNumber(row.strikes)}</td>
                    <td data-label={t('admin-probe-col-last')}>{row.reason === 'admin discovery from unknown country' ? t('admin-probe-reason-geo') : reason(row.lastSignal)}</td>
                    <td><Show when={canWrite()}><button class="btn btn--ghost btn--sm" aria-label={t('admin-probe-unblock-target', { source: row.source })} onClick={() => setUnblocking(row)}>{t('admin-probe-unblock')}</button></Show></td>
                  </tr>
                )}</For></tbody>
              </table>
            </Show>

            <h4 class={admin.heading}>{t('admin-probe-offenders-title')}</h4>
            <Show when={d().topOffenders.length > 0} fallback={<p class={admin.note}>{t('admin-probe-offenders-empty')}</p>}>
              <table class={pg.table} data-testid="admin-probe-offenders">
                <caption class={css.srCaption}>{t('admin-probe-offenders-title')}</caption>
                <thead><tr><th scope="col">{t('admin-probe-col-source')}</th><th scope="col">{t('admin-probe-col-score')}</th><th scope="col" class="num">{t('admin-probe-col-misses')}</th><th scope="col">{t('admin-probe-col-reason')}</th><th scope="col">{t('admin-probe-col-state')}</th></tr></thead>
                <tbody><For each={d().topOffenders}>{(row) => (
                  <tr data-fired={row.blocked ? 'true' : 'false'}>
                    <th scope="row"><code>{row.source}</code></th>
                    <td data-label={t('admin-probe-col-score')}>{t('admin-probe-score', { score: row.score, grace: row.graceAllowance })}</td>
                    <td class="num" data-label={t('admin-probe-col-misses')}>{formatNumber(row.signals)}</td>
                    <td data-label={t('admin-probe-col-reason')}>{reason(row.lastSignal)}</td>
                    <td data-label={t('admin-probe-col-state')}>{row.blocked ? t('admin-probe-state-blocked') : t('admin-probe-state-watched', { last: when(row.lastSeen) })}</td>
                  </tr>
                )}</For></tbody>
              </table>
            </Show>
          </div>

          <For each={PROBE_GROUPS}>{(group) => (
            <AdminSubsection title={t(`admin-probe-group-${group.id}`)} description={t(`admin-probe-group-help-${group.id}`)}>
              <Show when={group.probe.length > 0}>
                <ul class={css.rules}>
                  <For each={group.probe}>{(key) => probeRow(key, d())}</For>
                </ul>
              </Show>
              <Show when={group.registry.length > 0}>
                <Show when={canReadSettings()} fallback={<p class={admin.note}>{t('admin-protocol-panel-denied')}</p>}>
                  <Show when={settingsNow()?.kind === 'ok'} fallback={<Show when={settings.error || settingsNow() !== undefined}><p class={admin.note} role="alert">{t('admin-probe-settings-unavailable')}</p></Show>}>
                    <For each={group.registry.filter((k) => rowOf(k) !== undefined)}>{(key) => (
                      <div class={pg.editRow} data-testid={`admin-probe-geo-${key}`}>
                        <InlineSettingEditor settingKey={key} setting={rowOf(key)} onChanged={onSaved} />
                      </div>
                    )}</For>
                  </Show>
                </Show>
              </Show>
            </AdminSubsection>
          )}</For>
        </>}
      </Show>
      <ConfirmDialog open={confirmOn()} title={t('admin-probe-enable-title')} body={t('admin-probe-enable-body')} confirmLabel={t('admin-probe-enable-confirm')} danger={false} testId="admin-probe-enable-dialog"
        onCancel={() => setConfirmOn(false)} onConfirm={async () => { await save('enabled', true); setConfirmOn(false); }} />
      <ConfirmDialog open={unblocking() !== undefined} title={t('admin-probe-unblock-target', { source: unblocking()?.source ?? '' })} body={t('admin-probe-unblock-body')}
        confirmLabel={t('admin-probe-unblock')} onCancel={() => setUnblocking(undefined)} onConfirm={async () => { const row = unblocking(); if (row) await unblock(row); }} />
    </section>
  );
}

export function ProbeSummary(props: { policy: AdminProbePolicy }): JSX.Element {
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
