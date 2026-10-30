// t105: the tabs made of check cards: Server, Connectivity, Mail backends,
// Security posture and Tools.

import { For, Show, createMemo, createSignal, onMount, type JSX } from 'solid-js';
import { t, formatDateTime } from '../../../i18n';
import { formatMeasured } from '../Dashboard/format.ts';
import { describeAdminError } from '../shell/Feedback.tsx';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { useShell } from '../shell/context.ts';
import { clearAdminError, toastAdminError } from '../shell/requestErrorToast.ts';
import type { DnsReport, ProbeReport } from '../../../api/admin/domains.ts';
import type { Check, DetectResult, DiagnosticsApi, Run } from './api.ts';
import { CheckCard, CheckTable, type CardRow } from './Card.tsx';
import { KNOWN_CHECKS, backendFacts, cardStatus, groupOf, isKnownCheck, keyOf, latestChecks, latestRun, type CardStatus, type CheckGroup, type LatestCheck } from './model.ts';
import type { DiagnosticsStore, Suite } from './store.ts';
import * as css from './diagnostics.css.ts';

const when = (seconds: number): string => formatDateTime(new Date(seconds * 1000), { dateStyle: 'medium', timeStyle: 'medium' });
export const duration = (check: Check): string => check.status === 'skipped' || check.durationUs === null ? '—' : formatMeasured(check.durationMs, check.durationUs);
/** A row whose value is copyable when there is text to copy. */
const monoRow = (label: string, value: string, mono: string): CardRow => (mono ? { label, value, mono } : { label, value });
/** DNS record type names are not translated. */
const SRV = 'SRV';
/** Posture checks whose server summary is a fixed claim (diagnostics_posture.rs deployment_summary). */
const POSTURE_CLAIMS = new Set(['security-https', 'security-secure-cookies', 'security-hsts', 'security-csrf', 'security-admin-factors', 'security-signin-pacing', 'security-credential-locks', 'security-geography', 'security-render-isolation']);
const yesNo = (value: boolean): string => t(value ? 'admin-diagx-yes' : 'admin-diagx-no');

/** The status a card of `suite` shows now (Running only on the card that asked, or on all during Run all). */
export function liveStatus(store: DiagnosticsStore, suite: Suite, key: string, saved: string | undefined): CardStatus {
  if (store.running().has(suite) && (store.trigger() === key || store.trigger() === 'all')) return 'running';
  return cardStatus(saved);
}

export function checkTitle(check: Check): string {
  return isKnownCheck(check.id) ? t(`admin-diagx-check-${check.id}`) : check.summary;
}

/** The cards of one server-suite group, in the known order, then any the server added. */
export function serverChecks(runs: readonly Run[], group: Exclude<CheckGroup, 'backends'>): { id: string; latest: LatestCheck | undefined }[] {
  const latest = latestChecks(runs);
  const known = KNOWN_CHECKS[group];
  const extra = [...latest.values()].filter((l) => l.run.suite === 'server' && !known.includes(l.check.id) && groupOf(l.check, 'server') === group).map((l) => l.check.id);
  // security-deployment only appears when the assessment could not be read.
  return [...known.filter((id) => id !== 'security-deployment' || latest.has(id)), ...extra].map((id) => ({ id, latest: latest.get(id) }));
}

function ServerCard(props: { store: DiagnosticsStore; id: string; latest: LatestCheck | undefined }): JSX.Element {
  const shell = useShell();
  const check = (): Check | undefined => props.latest?.check;
  const known = (): boolean => isKnownCheck(props.id);
  // No server run yet: Not run. A server run that did not report this check
  // (it does not apply here, e.g. HSTS on plain HTTP): Skipped.
  const neverRun = (): boolean => !latestRun(props.store.runs(), 'server');
  const rows = (): CardRow[] => {
    const c = check();
    if (!c) return [{ label: t('admin-diagx-row-result'), value: t(neverRun() ? 'admin-diagx-not-run-yet' : 'admin-diagx-not-reported') }];
    // A posture check's summary states the goal whatever the status; say whether it is met.
    const result = POSTURE_CLAIMS.has(c.id) && (c.status === 'pass' || c.status === 'warn') ? t(c.status === 'pass' ? 'admin-diagx-posture-met' : 'admin-diagx-posture-not-met') : c.summary;
    return [
      { label: t('admin-diagx-row-result'), value: result },
      { kind: 'duration', label: t('admin-diagx-row-duration'), value: duration(c) },
      { label: t('admin-diagx-row-checked'), value: <time dateTime={new Date(props.latest!.run.startedAt * 1000).toISOString()}>{when(props.latest!.run.startedAt)}</time> },
      { label: t('admin-diagx-row-id'), value: c.id, mono: c.id },
    ];
  };
  return <CheckCard
    check={props.id}
    title={known() ? t(`admin-diagx-check-${props.id}`) : check()?.summary ?? props.id}
    purpose={known() ? t(`admin-diagx-check-${props.id}-purpose`) : t('admin-diagx-check-other-purpose')}
    status={liveStatus(props.store, 'server', props.id, check()?.status ?? (neverRun() ? undefined : 'skipped'))}
    rows={rows()}
    fix={known() ? t(`admin-diagx-check-${props.id}-fix`) : t('admin-diagx-check-other-fix')}
    raw={check()}
    onRun={() => void props.store.runSuite('server', props.id)}
    runLabel={check() ? t('admin-diagx-run-again') : t('admin-diagx-run')}
    runDisabled={!shell.can.can('diagnostics.run') || props.store.running().has('server')}
    runTitle={t('admin-diagx-run-server-note')}
  />;
}

/** A tab of server-suite cards (Server, Connectivity, Security posture). */
export function ServerGroup(props: { store: DiagnosticsStore; group: Exclude<CheckGroup, 'backends'> }): JSX.Element {
  // Keyed by check id, so a new run updates each card in place instead of re-creating it.
  const items = createMemo(() => serverChecks(props.store.runs(), props.group));
  const ids = createMemo(() => items().map((i) => i.id), undefined, { equals: (a, b) => a.length === b.length && a.every((id, n) => id === b[n]) });
  const latest = (id: string): LatestCheck | undefined => items().find((i) => i.id === id)?.latest;
  return <CheckTable caption={t(`admin-diagx-tab-${props.group}`)}><For each={ids()}>{(id) => <ServerCard store={props.store} id={id} latest={latest(id)} />}</For></CheckTable>;
}

// ── Mail backends ─────────────────────────────────────────────────────────────

function BackendCard(props: { store: DiagnosticsStore; check: Check }): JSX.Element {
  const shell = useShell();
  const facts = () => backendFacts(props.check);
  const endpoint = (): boolean => props.check.category === 'backend' || props.check.backendId !== undefined;
  const rows = (): CardRow[] => {
    const c = props.check;
    const f = facts();
    const out: CardRow[] = [{ label: t('admin-diagx-row-result'), value: c.summary }];
    if (c.domain) out.push({ label: t('admin-diagx-row-domain'), value: <a href={hrefFor(sectionPath('domains', [c.domain, 'connections']))}>{c.domain}</a>, mono: c.domain });
    if (c.backendId) out.push({ label: t('admin-diagx-row-backend'), value: c.backendId, mono: c.backendId });
    if (c.protocol) out.push({ label: t('admin-diagx-row-protocol'), value: c.protocol.toUpperCase(), mono: c.protocol });
    if (f.tls !== undefined || f.tlsVersion !== undefined) out.push({ label: t('admin-diagx-row-tls'), value: [f.tls, f.tlsVersion].filter(Boolean).join(' · '), mono: [f.tls, f.tlsVersion].filter(Boolean).join(' ') });
    if (f.capabilityCount !== undefined) out.push({ label: t('admin-diagx-row-capabilities'), value: String(f.capabilityCount) });
    if (f.credentialsSent !== undefined) out.push({ label: t('admin-diagx-row-credentials'), value: yesNo(f.credentialsSent) });
    out.push({ kind: 'duration', label: t('admin-diagx-row-duration'), value: duration(c) });
    return out;
  };
  const key = (): string => keyOf(props.check, 'backends');
  return <CheckCard
    check={key()}
    title={endpoint() ? t('admin-diagx-backend-title', { domain: props.check.domain ?? '', backend: props.check.backendId ?? '' }) : props.check.summary}
    purpose={endpoint() ? t('admin-diagx-backend-purpose', { protocol: (props.check.protocol ?? '').toUpperCase() }) : t('admin-diagx-backend-other-purpose')}
    status={liveStatus(props.store, 'backends', key(), props.check.status)}
    rows={rows()}
    fix={endpoint() ? t('admin-diagx-backend-fix') : props.check.fixId === 'domain-configuration' ? t('admin-diagx-backend-config-fix') : undefined}
    raw={props.check}
    onRun={props.store.canRunBackends() ? () => void props.store.runSuite('backends', key()) : undefined}
    runLabel={t('admin-diagx-run-again')}
    runDisabled={!shell.can.can('diagnostics.run') || props.store.running().has('backends')}
    runTitle={t('admin-diagx-run-backends-note')}
  />;
}

export function BackendsTab(props: { store: DiagnosticsStore; api: DiagnosticsApi }): JSX.Element {
  const shell = useShell();
  const [query, setQuery] = createSignal('');
  const [status, setStatus] = createSignal('');
  const [domains, setDomains] = createSignal<{ name: string; enabled: boolean; failover: boolean; backends: number }[]>();
  onMount(() => { void props.api.domains?.().then(setDomains).catch(() => setDomains([])); });
  const run = createMemo(() => props.store.job()?.run ?? latestRun(props.store.runs(), 'backends'));
  const results = createMemo(() => (run()?.results ?? []).filter((c) =>
    (!status() || cardStatus(c.status) === status())
    && `${c.summary} ${c.domain ?? ''} ${c.backendId ?? ''} ${c.protocol ?? ''}`.toLowerCase().includes(query().toLowerCase())));
  const job = () => props.store.job();
  return <div class={css.page}>
    <div class={css.toolbar}>
      <Show when={props.api.start}>
        <button class="btn btn--primary" type="button" disabled={!shell.can.can('diagnostics.run') || !props.store.canRunBackends() || props.store.running().has('backends')} title={t('admin-diag-backends-permission')} onClick={() => void props.store.runSuite('backends', 'tab')}>{t('admin-diag-backends')}</button>
      </Show>
      <Show when={job()?.run.status === 'running'}>
        <button class="btn" type="button" onClick={() => void props.store.cancel()}>{t('admin-diag-cancel')}</button>
        <span role="status">{t('admin-diagx-backends-progress', { completed: job()!.completed, total: job()!.total })}</span>
      </Show>
    </div>
    <Show when={!props.store.canRunBackends()}><p class={css.intro}>{t('admin-diag-backends-permission')}</p></Show>
    <section class={css.group} aria-labelledby="diag-failover">
      <h3 id="diag-failover" class={css.sectionTitle}>{t('admin-diagx-failover-title')}</h3>
      <p class={css.intro}>{t('admin-diagx-failover-purpose')}</p>
      <Show when={domains()} fallback={<p class={css.intro}>{t('admin-diagx-loading')}</p>}>
        <Show when={domains()!.length} fallback={<p class={css.intro}>{t('admin-diagx-no-domains')}</p>}>
          <dl class={css.rows} data-testid="diag-failover">
            <For each={domains()}>{(d) => <>
              <dt><a href={hrefFor(sectionPath('domains', [d.name, 'connections']))}>{d.name}</a></dt>
              <dd>{t('admin-diagx-failover-state', { state: d.failover ? 'on' : 'off', backends: d.backends, enabled: d.enabled ? 'yes' : 'no' })}</dd>
            </>}</For>
          </dl>
        </Show>
      </Show>
    </section>
    <section class={css.group} aria-labelledby="diag-backend-results">
      <h3 id="diag-backend-results" class={css.sectionTitle}>{t('admin-diagx-backend-results')}</h3>
      <Show when={run()} fallback={<p class={css.intro}>{t('admin-diagx-backends-not-run')}</p>}>{(r) => <>
        <p class={css.intro}>{t('admin-diagx-backends-run-at', { date: when(r().startedAt), count: r().results.length })}</p>
        <div class={css.filters}>
          <label>{t('admin-diag-filter')}<input type="search" value={query()} onInput={(e) => setQuery(e.currentTarget.value)} /></label>
          <label>{t('admin-diag-status')}<select value={status()} onChange={(e) => setStatus(e.currentTarget.value)}><option value="">{t('admin-diag-all')}</option><For each={['ok', 'warn', 'fail', 'skipped']}>{(value) => <option value={value}>{t(`admin-diagx-status-${value}`)}</option>}</For></select></label>
        </div>
        <CheckTable caption={t('admin-diagx-backend-results')}><For each={results().map((c) => c.id)} fallback={<tr><td colSpan={5}>{t(r().results.length ? 'admin-diag-no-match' : 'admin-diag-no-checks')}</td></tr>}>{(id) => <Show when={results().find((c) => c.id === id)}>{(check) => <BackendCard store={props.store} check={check()} />}</Show>}</For></CheckTable>
      </>}</Show>
    </section>
  </div>;
}

// ── Connectivity: per-domain tools ────────────────────────────────────────────

type ToolState<T> = { status: CardStatus; result?: T; error?: string };
function useTool<T>(key: string, run: () => Promise<T>, judge: (value: T) => CardStatus): [() => ToolState<T>, () => Promise<void>] {
  const [state, setState] = createSignal<ToolState<T>>({ status: 'notRun' });
  async function go(): Promise<void> {
    if (state().status === 'running') return;
    setState({ ...state(), status: 'running' });
    try { const value = await run(); setState({ status: judge(value), result: value }); }
    catch (e) {
      setState({ status: 'fail', error: describeAdminError(e) });
      toastAdminError(e, { key: `diag-tool-${key}`, retry: go });
      return;
    }
    clearAdminError(`diag-tool-${key}`);
  }
  return [state, go];
}

function dnsStatus(r: DnsReport): CardStatus {
  if (r.mx.error || r.mx.records.length === 0) return 'fail';
  return r.spf.present && r.dmarc.present ? 'ok' : 'warn';
}
function probeStatus(r: ProbeReport): CardStatus {
  if (r.results.length === 0) return 'skipped';
  if (r.results.some((p) => !p.reachable)) return 'fail';
  return r.results.some((p) => typeof p.certDaysLeft === 'number' && p.certDaysLeft < 14) ? 'warn' : 'ok';
}

export function ConnectivityTools(props: { api: DiagnosticsApi; canDiscover: () => boolean }): JSX.Element {
  const shell = useShell();
  const [domains, setDomains] = createSignal<string[]>([]);
  const [domain, setDomain] = createSignal('');
  const [discover, setDiscover] = createSignal('');
  onMount(() => { void props.api.domains?.().then((list) => { setDomains(list.map((d) => d.name)); if (!domain() && list[0]) setDomain(list[0].name); }).catch(() => undefined); });
  const [dns, runDns] = useTool('dns', () => props.api.dns!(domain()), dnsStatus);
  const [probe, runProbe] = useTool('tls', () => props.api.probe!(domain()), probeStatus);
  const [detect, runDetect] = useTool<DetectResult>('detect', () => props.api.detect!(discover().trim() || domain()), () => 'ok');
  const canRun = (): boolean => shell.can.can('diagnostics.run') && domain() !== '';
  const records = (list: readonly string[]): string => list.length ? list.join('\n') : t('admin-diagx-none');
  const dnsRows = (): CardRow[] => {
    const r = dns().result;
    if (!r) return [{ label: t('admin-diagx-row-result'), value: dns().error ?? t('admin-diagx-not-run-yet') }];
    const mx = r.mx.records.map((m) => `${m.preference} ${m.host}`);
    return [
      monoRow('MX', r.mx.error ?? records(mx), mx.join('\n')),
      monoRow('SPF', r.spf.error ?? records(r.spf.records), r.spf.records.join('\n')),
      monoRow('DMARC', r.dmarc.error ?? records(r.dmarc.records), r.dmarc.records.join('\n')),
      { label: SRV, value: t('admin-diagx-srv-found', { found: r.srv.filter((s) => s.records.length).length, total: r.srv.length }) },
    ];
  };
  const probeRows = (): CardRow[] => {
    const r = probe().result;
    if (!r) return [{ label: t('admin-diagx-row-result'), value: probe().error ?? t('admin-diagx-not-run-yet') }];
    if (!r.results.length) return [{ label: t('admin-diagx-row-result'), value: t('admin-diagx-probe-none') }];
    return r.results.map((p) => ({
      label: p.protocol.toUpperCase(),
      value: t('admin-diagx-probe-line', { host: `${p.host}:${p.port}`, reachable: p.reachable ? 'yes' : 'no', tls: p.tlsVersion ?? p.tls, days: p.certDaysLeft ?? 'none', time: formatMeasured(p.latencyMs, p.latencyUs), error: p.error ?? 'none' }),
      mono: `${p.host}:${p.port}`,
    }));
  };
  const detectRows = (): CardRow[] => {
    const r = detect().result;
    if (!r) return [{ label: t('admin-diagx-row-result'), value: detect().error ?? t('admin-diagx-not-run-yet') }];
    return [
      { label: t('admin-diagx-row-domain'), value: r.name, mono: r.name },
      { label: t('admin-diagx-row-source'), value: r.source, mono: r.source },
      { label: t('admin-diagx-row-oauth'), value: yesNo(r.requiresOAuth) },
    ];
  };
  return <section class={css.group} aria-labelledby="diag-domain-tools">
    <h3 id="diag-domain-tools" class={css.sectionTitle}>{t('admin-diagx-domain-tools')}</h3>
    <p class={css.intro}>{t('admin-diagx-domain-tools-purpose')}</p>
    <div class={css.filters}>
      <label>{t('admin-diagx-domain')}<select value={domain()} onChange={(e) => setDomain(e.currentTarget.value)}>
        <Show when={!domains().length}><option value="">{t('admin-diagx-no-domains')}</option></Show>
        <For each={domains()}>{(name) => <option value={name}>{name}</option>}</For>
      </select></label>
    </div>
    <CheckTable caption={t('admin-diagx-domain-tools')}>
      <Show when={props.api.dns}><CheckCard check="tool-dns" title={t('admin-diagx-tool-dns')} purpose={t('admin-diagx-tool-dns-purpose')} status={dns().status} rows={dnsRows()} fix={t('admin-diagx-tool-dns-fix')} raw={dns().result} onRun={() => void runDns()} runDisabled={!canRun()} /></Show>
      <Show when={props.api.probe}><CheckCard check="tool-tls" title={t('admin-diagx-tool-tls')} purpose={t('admin-diagx-tool-tls-purpose')} status={probe().status} rows={probeRows()} fix={t('admin-diagx-tool-tls-fix')} raw={probe().result} onRun={() => void runProbe()} runDisabled={!canRun()} /></Show>
      <Show when={props.api.detect}>
        <CheckCard check="tool-detect" title={t('admin-diagx-tool-detect')} purpose={t('admin-diagx-tool-detect-purpose')} status={detect().status} rows={detectRows()} fix={t('admin-diagx-tool-detect-fix')} raw={detect().result} onRun={() => void runDetect()} runDisabled={!props.canDiscover() || (discover().trim() === '' && domain() === '')}>
          <div class={css.filters}><label>{t('admin-diagx-detect-name')}<input type="text" inputmode="url" autocomplete="off" placeholder={domain()} value={discover()} onInput={(e) => setDiscover(e.currentTarget.value)} /></label></div>
        </CheckCard>
      </Show>
    </CheckTable>
  </section>;
}

// ── Tools ─────────────────────────────────────────────────────────────────────

export function ToolsTab(props: { store: DiagnosticsStore }): JSX.Element {
  const cat = () => props.store.catalogue();
  const link = (section: string, rest: string[], label: string, purpose: string): JSX.Element =>
    <article class={css.card}><header class={css.cardHead}><div class={css.cardTitles}><h4 class={css.cardTitle}>{label}</h4><p class={css.purpose}>{purpose}</p></div></header><div class={css.cardActions}><a class="btn" href={hrefFor(sectionPath(section, rest))}>{t('admin-diagx-open')}</a></div></article>;
  return <div class={css.page}>
    <div class={css.cards}>
      {link('dashboard', [], t('admin-diagx-tool-health'), t('admin-diagx-tool-health-purpose'))}
      {link('diagnostics', ['security'], t('admin-deployment-title'), t('admin-diagx-tool-deployment-purpose'))}
      {link('integrations', ['ssh'], t('admin-diagx-tool-ssh'), t('admin-diagx-tool-ssh-purpose'))}
      {link('security', ['geography'], t('admin-diagx-tool-geo'), t('admin-diagx-tool-geo-purpose'))}
    </div>
    <Show when={cat()}>{(c) => <article class={css.card} data-check="catalogue">
      <header class={css.cardHead}><div class={css.cardTitles}><h4 class={css.cardTitle}>{t('admin-diag-catalogue')}</h4><p class={css.purpose}>{t('admin-diagx-catalogue-purpose')}</p></div></header>
      <dl class={css.rows}><For each={c().suites}>{(suite) => <><dt><span class={css.mono}>{suite.id}</span></dt><dd>{suite.supported ? suite.description : t('admin-diagx-suite-unsupported', { reason: suite.reason ?? '' })}</dd></>}</For></dl>
      <Show when={c().limits}>{(limits) => <p class={css.purpose}>{t('admin-diag-limits', { concurrency: limits().concurrency, checks: limits().checks, run: limits().runSeconds, check: limits().checkSeconds })}</p>}</Show>
    </article>}</Show>
  </div>;
}
