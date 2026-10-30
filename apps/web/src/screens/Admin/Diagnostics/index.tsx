// t105: the Diagnostics screen. Sub-tabs grouped by purpose, each deep-linkable
// at #/diagnostics/<tab>:
//   Overview        latest results by group, what needs attention, Run all
//   Server          database, migrations, server key
//   Connectivity    public name and port, backend ports, egress policy, DNS/TLS/discovery tools
//   Mail backends   saved endpoint handshakes (the backends job), failover state
//   Security posture deployment posture checks, plaintext backends, fail2ban address
//   Tools           service health, deployment assessment, SSH tunnel tests, geography, limits
//   History         saved runs and what changed between runs
//   Report          the redacted troubleshooting report
// #/diagnostics/security (older bookmarks) still opens the deployment assessment.
// Service health probes (mail sync, outbox, search, rendering) stay on the
// Dashboard's Health panel; Overview links there.

import { For, Show, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { t, formatDateTime, loadCatalog } from '../../../i18n';
import { useShell } from '../shell/context.ts';
import { SectionTabs } from '../shell/SectionTabs.tsx';
import { DataTable } from '../shell/DataTable.tsx';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { Icon } from '../../../components/Icon.tsx';
import { toast } from '../../../components/toast/index.ts';
import { clearAdminError, toastAdminError } from '../shell/requestErrorToast.ts';
import { DeploymentSecurity } from '../DeploymentSecurity.tsx';
import * as adminCss from '../admin.css.ts';
import { httpDiagnosticsApi, type DiagnosticsApi, type Run } from './api.ts';
import { StatusChip, copyText } from './Card.tsx';
import { BackendsTab, ConnectivityTools, ServerGroup, ToolsTab, checkTitle, duration, liveStatus, serverChecks } from './checks.tsx';
import { CHECK_GROUPS, DIAG_TABS, cardStatus, countStatuses, diffRuns, latestRun, previousOf, tabOf, worst, type CardStatus, type CheckGroup, type DiagTab } from './model.ts';
import { createDiagnosticsStore, type DiagnosticsStore } from './store.ts';
import * as css from './diagnostics.css.ts';

export type { DiagnosticsApi } from './api.ts';

const when = (seconds: number): string => formatDateTime(new Date(seconds * 1000), { dateStyle: 'medium', timeStyle: 'medium' });

export function Diagnostics(props: { api?: DiagnosticsApi }): JSX.Element {
  const shell = useShell();
  const [ready, setReady] = createSignal(false);
  onMount(() => { void loadCatalog('admin-diagnostics').finally(() => setReady(true)); });
  return <Show when={shell.router.route().rest[0] === 'security'} fallback={<Show when={ready()}><DiagnosticsTabs api={props.api ?? httpDiagnosticsApi} /></Show>}><DeploymentSecurity /></Show>;
}

function DiagnosticsTabs(props: { api: DiagnosticsApi }): JSX.Element {
  const shell = useShell();
  const store = createDiagnosticsStore(() => props.api);
  onCleanup(() => store.dispose());
  const tab = (): DiagTab => tabOf(shell.router.route().rest[0]);
  const items = DIAG_TABS.map((id) => ({ id, label: t(`admin-diagx-tab-${id}`), path: id === 'overview' ? sectionPath('diagnostics') : sectionPath('diagnostics', [id]) }));
  return <section class={css.section} aria-label={t('admin-diag-title')}>
    <h2 class={adminCss.heading}>{t('admin-diag-title')}</h2>
    <SectionTabs label={t('admin-diagx-tabs')} items={items} selected={tab()} navigate={(path) => void shell.router.navigate(path)} />
    <Feedback kind="info" message={store.notice()} />
    <Show when={store.unavailable()}><p class={css.intro}>{t('admin-diag-unavailable')}</p></Show>
    <div data-diag-tab={tab()}>
      <Show when={tab() === 'overview'}><Overview store={store} /></Show>
      <Show when={tab() === 'server'}><Tab intro={t('admin-diagx-tab-server-intro')}><ServerGroup store={store} group="server" /></Tab></Show>
      <Show when={tab() === 'connectivity'}><Tab intro={t('admin-diagx-tab-connectivity-intro')}><ServerGroup store={store} group="connectivity" /><ConnectivityTools api={props.api} canDiscover={store.canRunBackends} /></Tab></Show>
      <Show when={tab() === 'backends'}><Tab intro={t('admin-diagx-tab-backends-intro')}><BackendsTab store={store} api={props.api} /></Tab></Show>
      <Show when={tab() === 'posture'}><Tab intro={t('admin-diagx-tab-posture-intro')}>
        <div class={css.links}><a class="btn" href={hrefFor(sectionPath('diagnostics', ['security']))}>{t('admin-deployment-title')}</a></div>
        <ServerGroup store={store} group="posture" />
      </Tab></Show>
      <Show when={tab() === 'tools'}><Tab intro={t('admin-diagx-tab-tools-intro')}><ToolsTab store={store} /></Tab></Show>
      <Show when={tab() === 'history'}><Tab intro={t('admin-diagx-tab-history-intro')}><HistoryTab store={store} /></Tab></Show>
      <Show when={tab() === 'report'}><Tab intro={t('admin-diag-redaction')}><ReportTab store={store} api={props.api} /></Tab></Show>
    </div>
  </section>;
}

function Tab(props: { intro: string; children: JSX.Element }): JSX.Element {
  return <div class={css.page}><p class={css.intro}>{props.intro}</p>{props.children}</div>;
}

// ── Overview ──────────────────────────────────────────────────────────────────

interface GroupSummary { group: CheckGroup; statuses: CardStatus[]; attention: { title: string; status: CardStatus }[] }
function summarise(store: DiagnosticsStore): GroupSummary[] {
  const runs = store.runs();
  return CHECK_GROUPS.map((group) => {
    if (group === 'backends') {
      const run = latestRun(runs, 'backends');
      const results = run?.results ?? [];
      const statuses = store.running().has('backends') ? ['running' as const] : results.length ? results.map((c) => cardStatus(c.status)) : ['notRun' as const];
      return { group, statuses, attention: results.filter((c) => ['warn', 'fail', 'error'].includes(c.status)).map((c) => ({ title: c.domain ? t('admin-diagx-backend-title', { domain: c.domain, backend: c.backendId ?? '' }) : c.summary, status: cardStatus(c.status) })) };
    }
    const items = serverChecks(runs, group);
    const ran = !!latestRun(runs, 'server');
    const statuses = items.map((i) => liveStatus(store, 'server', i.id, i.latest?.check.status ?? (ran ? 'skipped' : undefined)));
    return { group, statuses, attention: items.filter((i) => ['warn', 'fail'].includes(i.latest?.check.status ?? '')).map((i) => ({ title: checkTitle(i.latest!.check), status: cardStatus(i.latest!.check.status) })) };
  });
}

function Overview(props: { store: DiagnosticsStore }): JSX.Element {
  const shell = useShell();
  const groups = createMemo(() => summarise(props.store));
  const progress = () => props.store.progress();
  const lastServer = () => latestRun(props.store.runs(), 'server');
  const lastBackends = () => latestRun(props.store.runs(), 'backends');
  const attention = createMemo(() => groups().flatMap((g) => g.attention.map((a) => ({ ...a, group: g.group }))));
  return <div class={css.page}>
    <p class={css.intro}>{t('admin-diagx-overview-intro')}</p>
    <div class={css.toolbar}>
      <button class="btn btn--primary" type="button" disabled={!shell.can.can('diagnostics.run') || !!progress() || props.store.running().size > 0} onClick={() => void props.store.runAll()}>
        <Icon name={progress() ? 'sync' : 'refresh'} /><span>{progress() ? t('admin-diagx-running') : t('admin-diagx-run-all')}</span>
      </button>
      <Show when={props.store.job()?.run.status === 'running'}><button class="btn" type="button" onClick={() => void props.store.cancel()}>{t('admin-diag-cancel')}</button></Show>
      <a class="btn btn--ghost" href={hrefFor(sectionPath('dashboard'))}>{t('admin-diagx-open-health')}</a>
    </div>
    <Show when={progress()}>{(p) => <div class={css.progress}>
      <progress class={css.bar} max={p().steps} value={p().step - 1 + (p().total ? (p().completed ?? 0) / p().total! : 0)} aria-label={t('admin-diagx-run-all')} />
      <span role="status" data-testid="diag-run-all-progress">{t('admin-diagx-run-all-step', { step: p().step, steps: p().steps, suite: p().suite })}<Show when={p().total}>{' '}{t('admin-diagx-backends-progress', { completed: p().completed ?? 0, total: p().total ?? 0 })}</Show></span>
    </div>}</Show>
    <div class={css.tiles}>
      <For each={groups()}>{(g) => {
        const counts = () => countStatuses(g.statuses);
        return <a class={css.tile} href={hrefFor(sectionPath('diagnostics', [g.group]))} data-group={g.group}>
          <span class={css.tileHead}><span class={css.tileTitle}>{t(`admin-diagx-tab-${g.group}`)}</span><StatusChip status={worst(g.statuses)} /></span>
          <span class={css.tileCounts}>{t('admin-diagx-counts', { ok: counts().ok, warn: counts().warn, fail: counts().fail, notRun: counts().notRun + counts().skipped })}</span>
        </a>;
      }}</For>
    </div>
    <section class={css.group} aria-labelledby="diag-attention">
      <h3 id="diag-attention" class={css.sectionTitle}>{t('admin-diagx-attention')}</h3>
      <Show when={attention().length} fallback={<p class={css.intro}>{t(lastServer() ? 'admin-diagx-attention-none' : 'admin-diagx-never-run')}</p>}>
        <ul class={css.attention}><For each={attention()}>{(a) => <li class={css.attentionItem}><StatusChip status={a.status} /><a href={hrefFor(sectionPath('diagnostics', [a.group]))}>{a.title}</a></li>}</For></ul>
      </Show>
    </section>
    <dl class={css.rows}>
      <dt>{t('admin-diagx-last-server')}</dt><dd>{lastServer() ? when(lastServer()!.startedAt) : t('admin-diagx-never')}</dd>
      <dt>{t('admin-diagx-last-backends')}</dt><dd>{lastBackends() ? when(lastBackends()!.startedAt) : t('admin-diagx-never')}</dd>
    </dl>
  </div>;
}

// ── History ───────────────────────────────────────────────────────────────────

function HistoryTab(props: { store: DiagnosticsStore }): JSX.Element {
  const [selectedId, setSelectedId] = createSignal<string>();
  const selected = (): Run | undefined => props.store.runs().find((r) => r.id === selectedId()) ?? props.store.runs()[0];
  const changes = createMemo(() => { const run = selected(); const prior = run && previousOf(props.store.runs(), run); return run && prior ? { prior, list: diffRuns(run, prior) } : undefined; });
  return <div class={css.page}>
    <div class={css.toolbar}><button class="btn" type="button" disabled={props.store.loading()} onClick={() => void props.store.reload()}>{t('admin-diag-refresh')}</button></div>
    <DataTable<Run> caption={t('admin-diag-history')} rows={props.store.runs()} rowKey={(entry) => entry.id} state={props.store.loading() && !props.store.loaded() ? 'loading' : 'ready'} emptyText={t('admin-diag-empty')}
      columns={[
        { id: 'started', header: t('admin-diag-started'), cell: (entry) => <time dateTime={new Date(entry.startedAt * 1000).toISOString()}>{when(entry.startedAt)}</time> },
        { id: 'suite', header: t('admin-diag-suite'), cell: (entry) => t(`admin-diagx-suite-${entry.suite === 'backends' ? 'backends' : 'server'}`) },
        { id: 'status', header: t('admin-diag-status'), cell: (entry) => <StatusChip status={cardStatus(entry.status)} /> },
        { id: 'checks', header: t('admin-diag-checks'), cell: (entry) => entry.results.length },
        { id: 'failed', header: t('admin-diag-failed'), cell: (entry) => entry.results.filter((check) => check.status === 'fail').length },
      ]}
      rowActions={(entry) => <button type="button" class="btn btn--ghost" aria-label={t('admin-diag-open-run', { date: when(entry.startedAt) })} aria-pressed={selected()?.id === entry.id} onClick={() => setSelectedId(entry.id)}><Icon name="eye" /></button>}
    />
    <Show when={props.store.cursor()}><div class={css.toolbar}><button class="btn" type="button" disabled={props.store.loading()} onClick={() => void props.store.more()}>{t('admin-diag-more')}</button></div></Show>
    <Show when={selected()}>{(run) => <section class={adminCss.card} aria-labelledby="diag-run-detail" data-run={run().id}>
      <div class={css.cardHead}><div class={css.cardTitles}><h3 id="diag-run-detail" class={css.cardTitle}>{t('admin-diagx-run-title', { date: when(run().startedAt), suite: run().suite === 'backends' ? 'backends' : 'server' })}</h3></div><StatusChip status={cardStatus(run().status)} /></div>
      <section class={css.group} aria-labelledby="diag-run-changes">
        <h4 id="diag-run-changes" class={css.sectionTitle}>{t('admin-diagx-changes')}</h4>
        <Show when={changes()} fallback={<p class={css.intro}>{t('admin-diagx-changes-first')}</p>}>{(c) =>
          <Show when={c().list.length} fallback={<p class={css.intro}>{t('admin-diagx-changes-none', { date: when(c().prior.startedAt) })}</p>}>
            <p class={css.intro}>{t('admin-diagx-changes-since', { date: when(c().prior.startedAt) })}</p>
            <ul class={css.changeList} data-testid="diag-changes"><For each={c().list}>{(change) => <li>{change.summary}: {t('admin-diagx-change', { before: change.before ?? 'none', after: change.after ?? 'none' })}</li>}</For></ul>
          </Show>}
        </Show>
      </section>
      <div class={adminCss.tableWrap}><table class={adminCss.table}>
        <caption>{t('admin-diag-checks')}</caption>
        <thead><tr><th scope="col">{t('admin-diag-check')}</th><th scope="col">{t('admin-diag-status')}</th><th scope="col">{t('admin-diag-duration')}</th></tr></thead>
        <tbody><For each={run().results} fallback={<tr><td colSpan={3}>{t('admin-diag-no-checks')}</td></tr>}>{(check) => <tr><th scope="row">{checkTitle(check)}<Show when={check.domain}><div>{check.domain} · {check.backendId} · {check.protocol}</div></Show></th><td><StatusChip status={cardStatus(check.status)} /></td><td>{duration(check)}</td></tr>}</For></tbody>
      </table></div>
    </section>}</Show>
  </div>;
}

// ── Report ────────────────────────────────────────────────────────────────────

function ReportTab(props: { store: DiagnosticsStore; api: DiagnosticsApi }): JSX.Element {
  // Redaction is on until the admin turns it off: a report is usually pasted somewhere.
  const [omitDomains, setOmitDomains] = createSignal(true);
  const [runId, setRunId] = createSignal<string>('');
  const [preview, setPreview] = createSignal<string>();
  const [busy, setBusy] = createSignal(false);
  const selected = (): Run | undefined => props.store.runs().find((r) => r.id === runId()) ?? props.store.runs()[0];
  async function build(): Promise<string> {
    const run = selected();
    const report = await props.api.report?.({ omitDomains: omitDomains(), selected: run?.id }) as Record<string, unknown> | undefined;
    // The server attaches the selected run when it is in saved history, already
    // with placeholders when domains are omitted. A run it does not hold is
    // attached here only when names may appear.
    const attached = report && 'selectedRun' in report ? {} : omitDomains() ? { selectedRunOmitted: true } : { selectedRun: run };
    return JSON.stringify({ ...report, ...attached }, null, 2);
  }
  async function act(kind: 'copy' | 'export' | 'preview'): Promise<void> {
    setBusy(true);
    try {
      const text = await build();
      if (kind === 'copy') { await navigator.clipboard.writeText(text); toast.success(t('admin-diag-copied'), { id: 'admin-diagnostics-copied' }); }
      else if (kind === 'export') {
        const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
        const link = document.createElement('a'); link.href = url; link.download = 'mailwoman-diagnostics.json'; link.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
      } else setPreview(text);
      clearAdminError('diag-report');
    } catch (e) { if (!toastAdminError(e, { key: 'diag-report', retry: () => act(kind) })) setPreview(describeAdminError(e)); }
    finally { setBusy(false); }
  }
  return <Show when={props.api.report} fallback={<p class={css.intro}>{t('admin-diag-unavailable')}</p>}>
    <article class={css.card} data-check="report">
      <header class={css.cardHead}><div class={css.cardTitles}><h4 class={css.cardTitle}>{t('admin-diagx-report-title')}</h4><p class={css.purpose}>{t('admin-diagx-report-purpose')}</p></div></header>
      <div class={css.filters}>
        <label>{t('admin-diagx-report-run')}<select value={selected()?.id ?? ''} onChange={(e) => { setRunId(e.currentTarget.value); setPreview(undefined); }}>
          <Show when={!props.store.runs().length}><option value="">{t('admin-diagx-none')}</option></Show>
          <For each={props.store.runs()}>{(r) => <option value={r.id}>{t('admin-diagx-run-title', { date: when(r.startedAt), suite: r.suite === 'backends' ? 'backends' : 'server' })}</option>}</For>
        </select></label>
      </div>
      <label class={adminCss.note}><input type="checkbox" checked={omitDomains()} onChange={(e) => { setOmitDomains(e.currentTarget.checked); setPreview(undefined); }} /> {t('admin-diag-omit-domains')}</label>
      <div class={css.cardActions}>
        <button class="btn btn--primary" type="button" disabled={busy()} onClick={() => void act('copy')}><Icon name="clone" /><span>{t('admin-diag-copy')}</span></button>
        <button class="btn" type="button" disabled={busy()} onClick={() => void act('export')}><Icon name="download" /><span>{t('admin-diag-export')}</span></button>
        <button class="btn btn--ghost" type="button" disabled={busy()} onClick={() => void act('preview')}><Icon name="eye" /><span>{t('admin-diagx-report-preview')}</span></button>
      </div>
      <Show when={preview()}>{(text) => <details class={css.details} open><summary>{t('admin-diagx-report-preview')}</summary><pre class={css.raw} data-testid="diag-report-preview">{text()}</pre><button type="button" class="btn btn--ghost" onClick={() => void copyText(text())}><Icon name="clone" /><span>{t('admin-diagx-copy-details')}</span></button></details>}</Show>
    </article>
  </Show>;
}
