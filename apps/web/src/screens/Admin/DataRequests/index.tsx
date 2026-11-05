// Admin > Data requests: the queue of data subject requests with a countdown to
// each due date, filters, stats, and per-request actions (assign, identity check,
// approve, reject, extend, complete, legal hold, run now) with the request's
// trail. What is allowed and how long it takes are the `dsr.*` settings.
import { For, Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t, formatDateTime, formatNumber } from '../../../i18n';
import {
  assignDataRequest, approveDataRequest, completeDataRequest, createDataRequestOnBehalf, exportDataRequestLog,
  extendDataRequest, holdDataRequest, listDataRequests, loadDataRequest, rejectDataRequest, releaseDataRequestHold,
  runDataRequest, verifyDataRequest, type AdminDataRequest, type DataRequestFilter, type DataRequestList, type TrailRow,
} from '../../../api/admin/dataRequests.ts';
import { REQUEST_KINDS, type RequestKind } from '../../../api/dataRequests.ts';
import { StatTile } from '../shell/StatTile.tsx';
import { ChartCard, StatsSection } from '../shell/ChartCard.tsx';
import { LineChart } from '../shell/LineChart.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { actionsFor, daysLeft, logFileName, onTimePercent } from './model.ts';
import * as css from '../admin.css.ts';

const when = (s: number): string => formatDateTime(s * 1000);
const STATUSES = ['open', 'awaiting_confirmation', 'awaiting_verification', 'awaiting_approval', 'queued', 'processing', 'scheduled', 'in_progress', 'blocked', 'failed', 'ready', 'downloaded', 'expired', 'completed', 'rejected', 'cancelled'] as const;

function Countdown(props: { request: AdminDataRequest }): JSX.Element {
  const left = (): number | null => daysLeft(props.request, Math.floor(Date.now() / 1000));
  return <Show when={left() !== null} fallback={<span>{t('admin-dr-answered')}</span>}>
    <span data-testid="dr-countdown" data-state={props.request.dueState}>
      {(left() ?? 0) < 0 ? t('admin-dr-overdue-by', { days: -(left() ?? 0) }) : t('admin-dr-days-left', { days: left() ?? 0 })}
    </span>
  </Show>;
}

function Detail(props: { id: string; onChange: () => void; onClose: () => void }): JSX.Element {
  const [data, { refetch }] = createResource(() => props.id, id => loadDataRequest(id));
  const [note, setNote] = createSignal('');
  const [days, setDays] = createSignal(30);
  const [assignee, setAssignee] = createSignal('');
  const [error, setError] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  async function act(work: () => Promise<unknown>): Promise<void> {
    setBusy(true); setError('');
    try { await work(); setNote(''); await refetch(); props.onChange(); } catch (e) { setError(describeAdminError(e)); } finally { setBusy(false); }
  }
  return <Show when={data()} fallback={<p role="status">{t('admin-dr-loading')}</p>}>{d => {
    const r = (): AdminDataRequest => d().request;
    const a = (): ReturnType<typeof actionsFor> => actionsFor(r());
    return <section class={css.card} aria-label={t('admin-dr-detail')} data-testid="dr-detail">
      <div class={css.headingRow}>
        <h3 class={css.subHeading}>{t(`dr-kind-${r().kind}`)} · {r().address}</h3>
        <button type="button" class="btn btn--ghost" onClick={props.onClose}>{t('admin-dr-close')}</button>
      </div>
      <Show when={r().legalHold.active}>
        <p class={css.note} role="status" data-testid="dr-hold">{t('admin-dr-hold', { source: t(`admin-dr-hold-${r().legalHold.source ?? 'request'}`), reason: r().legalHold.reason ?? '' })}</p>
      </Show>
      <Show when={r().status === 'blocked'}><p class={css.note} role="status">{t('admin-dr-blocked-note')}</p></Show>
      <dl>
        <dt>{t('admin-dr-col-status')}</dt><dd>{t(`dr-status-${r().status}`)}</dd>
        <dt>{t('admin-dr-col-due')}</dt><dd>{when(r().dueAt)} (<Countdown request={r()} />)<Show when={r().extensionDays > 0}> · {t('dr-extended', { days: r().extensionDays })}: {r().extensionReason}</Show></dd>
        <dt>{t('admin-dr-filed-by')}</dt><dd>{r().requestedBy === 'self' ? t('admin-dr-by-self') : r().requestedBy}</dd>
        <Show when={r().details}><dt>{t('dr-details')}</dt><dd>{r().details}</dd></Show>
        <Show when={r().verifiedAt}><dt>{t('admin-dr-verified')}</dt><dd>{when(r().verifiedAt ?? 0)} · {r().verifiedBy}</dd></Show>
        <Show when={r().graceUntil}><dt>{t('admin-dr-grace')}</dt><dd>{when(r().graceUntil ?? 0)}</dd></Show>
        <Show when={r().error}><dt>{t('admin-dr-error')}</dt><dd>{r().error}</dd></Show>
        <Show when={r().completion}>{c => <><dt>{t('admin-dr-completion')}</dt><dd><code class={css.mono}>{JSON.stringify(c())}</code></dd></>}</Show>
      </dl>
      <div class={css.grid}>
        <label>{t('admin-dr-assignee')}<input value={assignee() || r().assignee || ''} onInput={e => setAssignee(e.currentTarget.value)} /></label>
        <button type="button" class="btn" disabled={busy()} onClick={() => void act(() => assignDataRequest(r().id, (assignee() || r().assignee || '').trim() || null))}>{t('admin-dr-assign')}</button>
      </div>
      <label>{t('admin-dr-note')}<input value={note()} onInput={e => setNote(e.currentTarget.value)} data-testid="dr-note" /></label>
      <label>{t('admin-dr-extend-days')}<input type="number" min="1" value={days()} onInput={e => setDays(Number(e.currentTarget.value) || 1)} /></label>
      <div class={css.headingRow}>
        <Show when={a().verify}><button type="button" class="btn" disabled={busy() || !note().trim()} onClick={() => void act(() => verifyDataRequest(r().id, note()))}>{t('admin-dr-verify')}</button></Show>
        <Show when={a().approve}><button type="button" class="btn" disabled={busy()} onClick={() => void act(() => approveDataRequest(r().id, note()))}>{t('admin-dr-approve')}</button></Show>
        <Show when={a().reject}><button type="button" class="btn" disabled={busy() || !note().trim()} onClick={() => void act(() => rejectDataRequest(r().id, note()))}>{t('admin-dr-reject')}</button></Show>
        <Show when={a().extend}><button type="button" class="btn" disabled={busy() || !note().trim()} onClick={() => void act(() => extendDataRequest(r().id, days(), note()))}>{t('admin-dr-extend')}</button></Show>
        <Show when={a().complete}><button type="button" class="btn" disabled={busy() || !note().trim()} onClick={() => void act(() => completeDataRequest(r().id, note()))}>{t('admin-dr-complete')}</button></Show>
        <Show when={a().hold}><button type="button" class="btn" disabled={busy() || !note().trim()} onClick={() => void act(() => holdDataRequest(r().id, note()))}>{t('admin-dr-hold-set')}</button></Show>
        <Show when={a().release}><button type="button" class="btn" disabled={busy()} onClick={() => void act(() => releaseDataRequestHold(r().id))}>{t('admin-dr-hold-release')}</button></Show>
        <Show when={a().run}><button type="button" class="btn" disabled={busy()} onClick={() => void act(() => runDataRequest(r().id))}>{t('admin-dr-run')}</button></Show>
      </div>
      <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
      <h4 class={css.subHeading}>{t('admin-dr-trail')}</h4>
      <ol data-testid="dr-trail"><For each={d().trail}>{(e: TrailRow) => <li>{when(e.ts)} · {e.actor} · {t(`dr-event-${e.kind}`)}</li>}</For></ol>
    </section>;
  }}</Show>;
}

export function AdminDataRequests(): JSX.Element {
  onMount(() => { void loadCatalog('data-requests'); void loadCatalog('admin-data-requests'); });
  const [filter, setFilter] = createSignal<DataRequestFilter>({ status: 'open' });
  const [list, { refetch }] = createResource(filter, f => listDataRequests(f));
  const [selected, setSelected] = createSignal<string | null>(null);
  const [error, setError] = createSignal('');
  const [behalfAddress, setBehalfAddress] = createSignal('');
  const [behalfKind, setBehalfKind] = createSignal<RequestKind>('access');
  const [behalfDetails, setBehalfDetails] = createSignal('');
  const patch = (p: Partial<DataRequestFilter>): void => { const next: DataRequestFilter = { ...filter(), ...p }; setFilter(() => next); };
  async function exportLog(): Promise<void> {
    try {
      const csv = await exportDataRequestLog(filter());
      const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv' }));
      const a = document.createElement('a'); a.href = url; a.download = logFileName(new Date()); a.click(); URL.revokeObjectURL(url);
    } catch (e) { setError(describeAdminError(e)); }
  }
  async function onBehalf(): Promise<void> {
    setError('');
    try {
      const made = await createDataRequestOnBehalf({ address: behalfAddress().trim(), kind: behalfKind(), details: behalfDetails() });
      setBehalfAddress(''); setBehalfDetails(''); setSelected(made.id); await refetch();
    } catch (e) { setError(describeAdminError(e)); }
  }
  const stats = (l: DataRequestList): DataRequestList['stats'] => l.stats;
  return <section class={css.section} aria-labelledby="admin-dr-heading" data-testid="admin-data-requests">
    <h2 id="admin-dr-heading" class={css.heading}>{t('admin-dr-title')}</h2>
    <p class={css.note}>{t('admin-dr-intro')} <a href={hrefFor(sectionPath('settings'))}>{t('admin-dr-settings-link')}</a></p>
    <Show when={list()}>{l => <>
      <StatsSection title={t('admin-dr-stats')} kind="tiles" testId="dr-stats">
        <StatTile label={t('admin-dr-tile-open')} value={stats(l()).open} testId="dr-tile-open" />
        <StatTile label={t('admin-dr-tile-overdue')} value={stats(l()).overdue} testId="dr-tile-overdue" />
        <StatTile label={t('admin-dr-tile-soon')} value={stats(l()).dueSoon} testId="dr-tile-soon" />
        <StatTile label={t('admin-dr-tile-ontime')} value={onTimePercent(stats(l()))} format={n => `${formatNumber(n)}%`} unavailableReason={t('admin-dr-tile-none')} testId="dr-tile-ontime" />
        <StatTile label={t('admin-dr-tile-avg')} value={stats(l()).avgAnswerHours === null ? null : Math.round((stats(l()).avgAnswerHours ?? 0) * 10) / 10} unavailableReason={t('admin-dr-tile-none')} detail={t('admin-dr-hours')} testId="dr-tile-avg" />
      </StatsSection>
      <StatsSection title={t('admin-dr-charts')} kind="charts">
        <ChartCard testId="dr-chart-created" title={t('admin-dr-chart-created')} unit={t('admin-dr-chart-unit')}
          summary={<>{t('admin-dr-chart-summary', { total: stats(l()).created30d.reduce((n, p) => n + p.count, 0) })}</>}
          table={{ columns: [t('admin-dr-col-day'), t('admin-dr-col-count')], numeric: [false, true], rows: stats(l()).created30d.map(p => [formatDateTime(p.t * 1000, { dateStyle: 'short' }), String(p.count)]) }}
          empty={stats(l()).created30d.every(p => p.count === 0) ? t('admin-dr-chart-empty') : undefined}>
          <LineChart times={stats(l()).created30d.map(p => p.t)} series={[{ label: t('admin-dr-chart-created'), slot: 1, values: stats(l()).created30d.map(p => p.count) }]} format={n => formatNumber(n)} formatTime={s => formatDateTime(s * 1000, { dateStyle: 'short' })} whole />
        </ChartCard>
        <BarChart testId="dr-chart-status" title={t('admin-dr-chart-status')} bars={Object.entries(stats(l()).byStatus).map(([k, v]) => ({ label: t(`dr-status-${k}`), value: v }))} emptyText={t('admin-dr-chart-empty')} />
      </StatsSection>
    </>}</Show>
    <div class={css.grid} role="search" aria-label={t('admin-dr-filters')}>
      <label>{t('admin-dr-col-status')}<select value={filter().status ?? ''} onChange={e => patch({ status: e.currentTarget.value })}><option value="">{t('admin-dr-any')}</option><For each={STATUSES}>{s => <option value={s}>{s === 'open' ? t('admin-dr-open') : t(`dr-status-${s}`)}</option>}</For></select></label>
      <label>{t('admin-dr-col-type')}<select value={filter().kind ?? ''} onChange={e => patch({ kind: e.currentTarget.value })}><option value="">{t('admin-dr-any')}</option><For each={REQUEST_KINDS}>{k => <option value={k}>{t(`dr-kind-${k}`)}</option>}</For></select></label>
      <label>{t('admin-dr-col-due')}<select value={filter().due ?? ''} onChange={e => patch({ due: e.currentTarget.value })}><option value="">{t('admin-dr-any')}</option><option value="overdue">{t('admin-dr-due-overdue')}</option><option value="soon">{t('admin-dr-due-soon')}</option></select></label>
      <label>{t('admin-dr-assignee')}<input value={filter().assignee ?? ''} placeholder={t('admin-dr-assignee-hint')} onChange={e => patch({ assignee: e.currentTarget.value })} /></label>
      <label>{t('admin-dr-search')}<input type="search" value={filter().q ?? ''} onChange={e => patch({ q: e.currentTarget.value })} /></label>
      <button type="button" class="btn" onClick={() => void exportLog()}>{t('admin-dr-export')}</button>
    </div>
    <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
    <Show when={list()?.requests.length} fallback={<p class={css.note} role="status">{list.loading ? t('admin-dr-loading') : t('admin-dr-empty')}</p>}>
      <div class={css.tableWrap}><table class={css.table}>
        <caption>{t('admin-dr-caption', { shown: list()?.requests.length ?? 0, total: list()?.matched ?? 0 })}</caption>
        <thead><tr><th>{t('admin-dr-col-person')}</th><th>{t('admin-dr-col-type')}</th><th>{t('admin-dr-col-status')}</th><th>{t('admin-dr-col-due')}</th><th>{t('admin-dr-assignee')}</th><th><span class="sr-only">{t('admin-dr-col-actions')}</span></th></tr></thead>
        <tbody><For each={list()?.requests ?? []}>{r => <tr data-testid={`dr-row-${r.id}`}>
          <td>{r.address}</td><td>{t(`dr-kind-${r.kind}`)}</td>
          <td>{t(`dr-status-${r.status}`)}<Show when={r.legalHold.active}> <span class={css.badge}>{t('admin-dr-hold-badge')}</span></Show></td>
          <td><Countdown request={r} /></td><td>{r.assignee ?? t('admin-dr-unassigned')}</td>
          <td><button type="button" class="btn btn--ghost" onClick={() => setSelected(r.id)}>{t('admin-dr-open-request')}</button></td>
        </tr>}</For></tbody>
      </table></div>
    </Show>
    <Show when={selected()}>{id => <Detail id={id()} onChange={() => void refetch()} onClose={() => setSelected(null)} />}</Show>
    <section class={css.card} aria-label={t('admin-dr-behalf')}>
      <h3 class={css.subHeading}>{t('admin-dr-behalf')}</h3>
      <p class={css.note}>{t('admin-dr-behalf-help')}</p>
      <label>{t('admin-dr-col-person')}<input type="email" value={behalfAddress()} onInput={e => setBehalfAddress(e.currentTarget.value)} /></label>
      <label>{t('admin-dr-col-type')}<select value={behalfKind()} onChange={e => setBehalfKind(e.currentTarget.value as RequestKind)}><For each={REQUEST_KINDS}>{k => <option value={k}>{t(`dr-kind-${k}`)}</option>}</For></select></label>
      <label>{t('dr-details')}<input value={behalfDetails()} onInput={e => setBehalfDetails(e.currentTarget.value)} /></label>
      <button type="button" class="btn" disabled={!behalfAddress().trim()} onClick={() => void onBehalf()}>{t('admin-dr-behalf-submit')}</button>
    </section>
  </section>;
}

export default AdminDataRequests;
