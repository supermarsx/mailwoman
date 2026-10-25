import { For, Show, createSignal, createEffect, on, onMount, type JSX } from 'solid-js';
import { domainsApi, type DomainsApi, type DomainDetail, type ManagedDomain, type DomainUpstream, type DomainEndpoint, type DomainChange, type DomainPreview, type ProbeReport, type DnsReport, type DomainStatus } from '../../../api/admin/domains.ts';
import { loadCatalog, t } from '../../../i18n';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { useDirtyGuard } from '../shell/useDirtyGuard.ts';
import { sectionPath, hrefFor } from '../shell/router.ts';
import { useShell } from '../shell/context.ts';
import { AdminProblemError } from '../shell/adminApi.ts';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import * as css from '../admin.css.ts';

const lines = (text: string): string[] => [...new Set(text.split(/\r?\n/).map(s => s.trim()).filter(Boolean))];
const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);
type Protocol = 'imap' | 'pop3' | 'smtp' | 'sieve';
const endpoint = (protocol: Protocol): DomainEndpoint => ({ host: '', port: { imap: 993, pop3: 995, smtp: 465, sieve: 4190 }[protocol], tls: protocol === 'sieve' ? 'starttls' : 'implicit', auth: 'password', allowPrivate: false });

function EndpointFields(props: { protocol: Protocol; value: DomainEndpoint; disabled: boolean; update: (value: DomainEndpoint) => void }): JSX.Element {
  const change = (patch: Partial<DomainEndpoint>) => props.update({ ...props.value, ...patch });
  return <fieldset disabled={props.disabled} class={css.card}>
    <legend>{props.protocol.toUpperCase()}</legend>
    <label class="field"><span>{t('domain-editor-host')}</span><input aria-label={`${props.protocol.toUpperCase()} ${t('domain-editor-host')}`} required value={props.value.host} onInput={e => change({ host: e.currentTarget.value })} /></label>
    <label class="field"><span>{t('domain-editor-port')}</span><input aria-label={`${props.protocol.toUpperCase()} ${t('domain-editor-port')}`} required type="number" min="1" max="65535" value={props.value.port} onInput={e => change({ port: e.currentTarget.valueAsNumber })} /></label>
    <label class="field"><span>{t('domain-editor-tls')}</span><select aria-label={`${props.protocol.toUpperCase()} ${t('domain-editor-tls')}`} value={props.value.tls} onChange={e => change({ tls: e.currentTarget.value as DomainEndpoint['tls'] })}><option value="implicit">{t('domain-editor-implicit')}</option><option value="starttls">STARTTLS</option><option value="none">{t('domain-editor-no-tls')}</option></select></label>
    <Show when={props.value.tls === 'none'}><p role="note">{t('domain-editor-plaintext-warning')}</p></Show>
    <label class="field"><span>{t('domain-editor-auth')}</span><select aria-label={`${props.protocol.toUpperCase()} ${t('domain-editor-auth')}`} value="password" disabled><option value="password">{t('domain-editor-password')}</option><option value="oauth" disabled>{t('domain-editor-oauth-unsupported')}</option></select></label>
    <label><input type="checkbox" checked={props.value.allowPrivate ?? false} onChange={e => change({ allowPrivate: e.currentTarget.checked })} /> {t('domain-editor-private', { protocol: props.protocol.toUpperCase() })}</label>
  </fieldset>;
}

export function Domains(props: { service?: DomainsApi }): JSX.Element {
  const api = props.service ?? domainsApi;
  const shell = useShell();
  const [status, setStatus] = createSignal<DomainStatus>();
  const [rows, setRows] = createSignal<ManagedDomain[]>([]);
  const [selected, setSelected] = createSignal<DomainDetail>();
  const [capabilities, setCapabilities] = createSignal<string[]>([]);
  const [name, setName] = createSignal('');
  const [upstream, setUpstream] = createSignal<DomainUpstream>({});
  const [allow, setAllow] = createSignal('');
  const [block, setBlock] = createSignal('');
  const [discovery, setDiscovery] = createSignal('off');
  const [alias, setAlias] = createSignal('');
  const [host, setHost] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [ready, setReady] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal<string | null>(null);
  const [preview, setPreview] = createSignal<{ result: DomainPreview; changes: DomainChange }>();
  const [confirmation, setConfirmation] = createSignal<'delete' | 'disable' | 'switch' | 'reload'>();
  const [switchName, setSwitchName] = createSignal('');
  const [force, setForce] = createSignal(false);
  const [probe, setProbe] = createSignal<ProbeReport>();
  const [dns, setDns] = createSignal<DnsReport>();
  const [proposal, setProposal] = createSignal<Awaited<ReturnType<DomainsApi['detect']>>>();
  const can = (cap: string) => capabilities().includes(cap);
  const canWrite = () => can('domains.write');
  const canUpstream = () => canWrite() && can('domains.upstream_write');
  const canDiagnose = () => canUpstream() && can('diagnostics.run');
  const canPolicy = () => canWrite() && can('domains.signin_policy');
  const change = (): DomainChange => {
    const original = selected(); if (!original) return {};
    const result: DomainChange = {};
    if (!same(original.upstream, upstream())) result.upstream = upstream();
    if (!same(original.allowlist, lines(allow()))) result.allowlist = lines(allow());
    if (!same(original.blocklist, lines(block()))) result.blocklist = lines(block());
    if ((original.settings.discovery ?? 'off') !== discovery()) result.settings = { ...original.settings, discovery: discovery() };
    return result;
  };
  const dirty = () => Object.keys(change()).length > 0;
  useDirtyGuard(dirty);
  createEffect(on(() => [ready(), shell.router.route().rest[0]] as const, ([loaded, target]) => { if (!loaded || !target) return; if (rows().some(row => row.name === target)) void run(() => load(target)); else setName(target); }));
  function populate(d: DomainDetail): void { setSelected(d); setUpstream(structuredClone(d.upstream ?? {})); setAllow((d.allowlist ?? []).join('\n')); setBlock((d.blocklist ?? []).join('\n')); setDiscovery(typeof d.settings.discovery === 'string' ? d.settings.discovery : 'off'); setPreview(undefined); setProposal(undefined); setProbe(undefined); setDns(undefined); setAlias(''); setHost(''); }
  function summary(key: string, value: ManagedDomain | null): string {
    if (!value) return t('domain-editor-none');
    if (key === 'allowlist' || key === 'blocklist') return value[key].join(', ') || t('domain-editor-none');
    if (key === 'settings') return value.settings.discovery === 'detect-on-request' ? t('domain-editor-discovery-request') : t('domain-editor-discovery-off');
    return Object.entries(value.upstream ?? {}).filter(([, endpoint]) => endpoint != null).map(([protocol, endpoint]) => `${protocol.toUpperCase()}: ${'host' in endpoint ? `${endpoint.host}:${endpoint.port} (${endpoint.tls}${endpoint.allowPrivate ? ', ' + t('domain-editor-private-summary') : ''})` : 'sessionUrl' in endpoint ? endpoint.sessionUrl : [endpoint.caldavUrl, endpoint.carddavUrl].filter(Boolean).join(', ')}`).join('; ') || t('domain-editor-default');
  }
  async function load(name: string): Promise<void> { populate(await api.detail(name)); }
  async function refresh(): Promise<void> { const [rows, status] = await Promise.all([api.list(), api.status()]); setRows(rows); setStatus(status); if (selected()) await load(selected()!.name); }
  function errorText(e: unknown): string { return e instanceof AdminProblemError && e.stale ? t('domain-editor-stale') : e instanceof AdminProblemError && e.detail ? e.detail : describeAdminError(e); }
  async function run(action: () => Promise<unknown>, announce = false): Promise<void> {
    if (busy()) return; setBusy(true); setError(null); setSuccess(null);
    try { await action(); if (announce) setSuccess(t('domain-editor-saved')); }
    catch (e) { setError(errorText(e)); }
    finally { setBusy(false); }
  }
  function select(name: string): void { if (dirty()) { setSwitchName(name); setConfirmation('switch'); } else void run(() => load(name)); }
  async function create(event: Event): Promise<void> { event.preventDefault(); const value = name().trim(); if (!value) return; await run(async () => { await api.save(value, {}, null); setName(''); setRows(await api.list()); await load(value); }, true); }
  function mail(protocol: string): void { const next = { ...upstream() }; delete next.imap; delete next.pop3; if (protocol === 'imap' || protocol === 'pop3') next[protocol] = endpoint(protocol); setUpstream(next); }
  function toggle(protocol: 'smtp' | 'sieve', checked: boolean): void { const next = { ...upstream() }; if (checked) next[protocol] = endpoint(protocol); else delete next[protocol]; setUpstream(next); }
  function dav(key: 'caldavUrl' | 'carddavUrl', value: string): void { const next = { ...upstream(), dav: { ...upstream().dav } }; if (value.trim()) next.dav[key] = value; else delete next.dav[key]; if (Object.keys(next.dav).length === 0) delete (next as DomainUpstream).dav; setUpstream(next); }
  async function save(event: Event): Promise<void> { event.preventDefault(); const d = selected(); if (!d || !dirty()) return; const changes = structuredClone(change()); await run(async () => { const result = await api.save(d.name, { ...changes, dryRun: true }, d.etag) as DomainPreview; if (result.dryRun !== true || !result.after) throw new Error(t('domain-editor-preview-error')); setPreview({ result, changes }); }); }
  async function mapping(kind: 'aliases' | 'hosts', value: string, remove = false): Promise<void> { const d = selected(); if (!d || !value.trim()) return; await run(async () => { await api.mapping(d.name, kind, value.trim(), remove, d.mappingEtag); const fresh = await api.detail(d.name); setSelected({ ...d, aliases: fresh.aliases, hosts: fresh.hosts, mappingEtag: fresh.etag === d.etag ? fresh.mappingEtag : d.mappingEtag }); if (kind === 'aliases') setAlias(''); else setHost(''); }, true); }
  onMount(() => { void run(async () => { await loadCatalog('admin-domains'); const [data, caps, status] = await Promise.all([api.list(), api.capabilities(), api.status()]); setRows(data); setCapabilities(caps); setStatus(status); }).then(() => setReady(true)); });
  return <section class={css.section} aria-label={t('admin-domains-title')}>
    <h2 class={css.heading}>{t('admin-domains-title')}</h2>
    <Feedback kind="error" message={error()} /><Feedback kind="success" message={success()} />
    <Show when={ready()}>
      <p class={css.note}>{t('domain-editor-intro')}</p>
      <Show when={status()?.blockedAccounts.total}><section class={css.card} aria-label={t('domain-editor-blocked-title')}><h3>{t('domain-editor-blocked-title')}</h3><p>{t('domain-editor-blocked-count', { count: status()!.blockedAccounts.total })}</p><For each={status()!.blockedAccounts.byDomain}>{row => <div class={css.listRow}><div><strong>{row.domain}</strong><p>{t('domain-editor-blocked-domain-count', { count: row.accounts })}</p><Show when={row.seenHost}><p>{t('domain-editor-import-host', { host: row.seenHost! })}</p></Show></div><Show when={canWrite()}><a class="btn" href={hrefFor(sectionPath('domains', [row.domain]))} onClick={e => { e.preventDefault(); void shell.router.navigate(sectionPath('domains', [row.domain])); }}>{t('domain-editor-configure', { name: row.domain })}</a></Show></div>}</For></section></Show>
      <Show when={canWrite()}><form class={css.card} aria-label={t('admin-domains-add')} onSubmit={e => void create(e)}><label class="field"><span>{t('admin-domains-name')}</span><input required placeholder="example.com" value={name()} disabled={busy()} onInput={e => setName(e.currentTarget.value)} /></label><button class="btn btn--primary" disabled={busy()}>{t('domain-editor-create')}</button></form></Show>
      <div class={css.card}><Show when={rows().length} fallback={<p>{t('admin-domains-empty')}</p>}><For each={rows()}>{d => <div class={css.listRow}><button class="btn btn--ghost" disabled={busy()} aria-pressed={selected()?.name === d.name} onClick={() => select(d.name)}>{d.name}</button><span>{d.enabled ? t('domain-editor-enabled') : t('domain-editor-disabled')}{d.preferred ? ` · ${t('domain-editor-preferred')}` : ''}</span></div>}</For></Show></div>
      <Show when={selected()}>{d => <>
        <h3>{d().name}</h3><p>{t('domain-editor-mode', { mode: d().mode })}</p>
        <div class={css.listRow}>
          <button type="button" class="btn" disabled={busy()} onClick={() => dirty() ? setConfirmation('reload') : void run(refresh)}>{t('domain-editor-reload')}</button>
          <Show when={canWrite()}><button class="btn" disabled={busy() || dirty()} onClick={() => d().enabled ? setConfirmation('disable') : void run(async () => { await api.enable(d().name, true, d().etag); await refresh(); }, true)}>{d().enabled ? t('domain-editor-disable') : t('domain-editor-enable')}</button><button class="btn" disabled={busy() || dirty()} onClick={() => void run(async () => { if (d().preferred) await api.clearPreferred(d().preferenceEtag); else await api.prefer(d().name, d().preferenceEtag); await refresh(); }, true)}>{d().preferred ? t('domain-editor-clear-preferred') : t('domain-editor-set-preferred')}</button></Show>
          <Show when={can('domains.delete')}><button class="btn" disabled={busy()} onClick={() => { setForce(false); setConfirmation('delete'); }}>{t('admin-delete')}</button></Show>
        </div>
        <Show when={canDiagnose()}><section class={css.card} aria-label={t('domain-editor-diagnostics')}><h4>{t('domain-editor-diagnostics')}</h4><p>{t('domain-editor-probe-help')}</p><Show when={dirty()}><p>{t('domain-editor-save-before-probe')}</p></Show><button type="button" class="btn" disabled={busy() || dirty()} onClick={() => void run(async () => setProbe(await api.probe(d().name)))}>{t('domain-editor-test-connection')}</button> <button type="button" class="btn" disabled={busy() || dirty()} onClick={() => void run(async () => setDns(await api.dns(d().name)))}>{t('domain-editor-test-dns')}</button>
          <Show when={probe()}>{report => <div><p>{t('domain-editor-probe-ran', { time: report().ranAt })}</p><Show when={report().results.length === 0}><p>{t('domain-editor-no-endpoints')}</p></Show><For each={report().results}>{result => <article class={css.card}><h5>{result.protocol.toUpperCase()} — {result.host}:{result.port}</h5><p>{result.reachable ? t('domain-editor-reachable') : t('domain-editor-unreachable')} · {result.latencyMs} ms</p><Show when={result.error}><p role="alert">{result.error}</p></Show><p>{t('domain-editor-tls')}: {result.tlsVersion ?? (result.tls === 'none' ? t('domain-editor-no-tls') : t('domain-editor-tls-unavailable'))}</p><Show when={result.certNotAfter}><p>{t('domain-editor-cert-expiry', { date: result.certNotAfter!, days: result.certDaysLeft ?? 0 })}</p></Show><Show when={result.greeting}><p>{result.greeting}</p></Show><Show when={result.capabilities.length}><details><summary>{t('domain-editor-capabilities')}</summary><ul><For each={result.capabilities}>{cap => <li>{cap}</li>}</For></ul></details></Show></article>}</For></div>}</Show>
          <Show when={dns()}>{report => <div><h5>MX</h5><Show when={report().mx.error}><p role="alert">{report().mx.error}</p></Show><For each={report().mx.records}>{record => <p>{record.preference} · {record.host}</p>}</For><Show when={!report().mx.error && report().mx.records.length === 0}><p>{t('domain-editor-dns-absent')}</p></Show><For each={(['spf', 'dmarc'] as const)}>{kind => <div><h5>{kind.toUpperCase()}</h5><Show when={report()[kind].error}><p role="alert">{report()[kind].error}</p></Show><For each={report()[kind].records}>{record => <p>{record}</p>}</For><Show when={!report()[kind].error && !report()[kind].present}><p>{t('domain-editor-dns-absent')}</p></Show></div>}</For><For each={report().srv}>{service => <div><h5>{service.service}</h5><Show when={service.error}><p role="alert">{service.error}</p></Show><For each={service.records}>{record => <p>{record.host}:{record.port} · {t('domain-editor-srv-order', { priority: record.priority, weight: record.weight })}</p>}</For><Show when={!service.error && service.records.length === 0}><p>{t('domain-editor-dns-absent')}</p></Show></div>}</For></div>}</Show>
        </section></Show>
        <form aria-label={t('domain-editor-configure', { name: d().name })} onSubmit={e => void save(e)}>
          <fieldset disabled={busy()}><legend>{t('domain-editor-upstream')}</legend>
            <Show when={d().mode === 'engine'} fallback={<label class="field"><span>{t('domain-editor-jmap')}</span><input type="url" disabled={!canUpstream()} value={upstream().jmap?.sessionUrl ?? ''} onInput={e => setUpstream(e.currentTarget.value.trim() ? { jmap: { sessionUrl: e.currentTarget.value } } : {})} /></label>}>
              <label class="field"><span>{t('domain-editor-mail-protocol')}</span><select disabled={!canUpstream()} value={upstream().imap ? 'imap' : upstream().pop3 ? 'pop3' : ''} onChange={e => mail(e.currentTarget.value)}><option value="">{t('domain-editor-default')}</option><option value="imap">IMAP</option><option value="pop3">POP3</option></select></label>
              <For each={(['imap', 'pop3'] as const)}>{protocol => <Show when={upstream()[protocol]}>{value => <EndpointFields protocol={protocol} value={value()} disabled={!canUpstream()} update={v => setUpstream({ ...upstream(), [protocol]: v })} />}</Show>}</For>
              <For each={(['smtp', 'sieve'] as const)}>{protocol => <><label><input type="checkbox" disabled={!canUpstream()} checked={!!upstream()[protocol]} onChange={e => toggle(protocol, e.currentTarget.checked)} /> {t('domain-editor-use-protocol', { protocol: protocol.toUpperCase() })}</label><Show when={upstream()[protocol]}>{value => <EndpointFields protocol={protocol} value={value()} disabled={!canUpstream()} update={v => setUpstream({ ...upstream(), [protocol]: v })} />}</Show></>}</For>
              <label class="field"><span>CalDAV URL</span><input type="url" disabled={!canUpstream()} value={upstream().dav?.caldavUrl ?? ''} onInput={e => dav('caldavUrl', e.currentTarget.value)} /></label>
              <label class="field"><span>CardDAV URL</span><input type="url" disabled={!canUpstream()} value={upstream().dav?.carddavUrl ?? ''} onInput={e => dav('carddavUrl', e.currentTarget.value)} /></label>
            </Show>
            <Show when={d().mode === 'proxy' && canUpstream() && (status()?.upstreamChoices.length ?? 0) > 0}><label class="field"><span>{t('domain-editor-upstream-choices')}</span><select value="" onChange={e => { if (e.currentTarget.value) setUpstream({ jmap: { sessionUrl: `${e.currentTarget.value.replace(/\/$/, '')}/.well-known/jmap` } }); }}><option value="">{t('domain-editor-choose-upstream')}</option><For each={status()!.upstreamChoices}>{origin => <option value={origin}>{origin}</option>}</For></select></label></Show>
            <Show when={canUpstream()}><button type="button" class="btn" onClick={() => void run(async () => setProposal(await api.detect(d().name)))}>{t('domain-editor-detect')}</button></Show>
            <Show when={proposal()}>{p => <div class={css.card}><p>{t('domain-editor-proposal', { source: p().source })}</p><For each={Object.entries(p().upstream).filter(([, endpoint]) => endpoint != null)}>{([protocol, value]) => <p>{protocol.toUpperCase()}: {'host' in value ? `${value.host}:${value.port}` : 'sessionUrl' in value ? value.sessionUrl : [value.caldavUrl, value.carddavUrl].filter(Boolean).join(', ')}</p>}</For><Show when={p().requiresOAuth}><p role="alert">{t('domain-editor-oauth-warning')}</p></Show><button type="button" class="btn" disabled={p().requiresOAuth} onClick={() => { setUpstream(structuredClone(p().upstream)); setProposal(undefined); }}>{t('domain-editor-apply-proposal')}</button></div>}</Show>
          </fieldset>
          <fieldset disabled={busy() || !canWrite()}><legend>{t('domain-editor-discovery')}</legend><select aria-label={t('domain-editor-discovery')} value={discovery()} onChange={e => setDiscovery(e.currentTarget.value)}><option value="off">{t('domain-editor-discovery-off')}</option><option value="detect-on-request">{t('domain-editor-discovery-request')}</option></select></fieldset>
          <fieldset disabled={busy() || !canPolicy()}><legend>{t('domain-editor-signin')}</legend><p>{t('domain-editor-policy-help')}</p><label class="field"><span>{t('domain-editor-allowlist')}</span><textarea rows="4" value={allow()} onInput={e => setAllow(e.currentTarget.value)} /></label><label class="field"><span>{t('domain-editor-blocklist')}</span><textarea rows="4" value={block()} onInput={e => setBlock(e.currentTarget.value)} /></label></fieldset>
          <Show when={canWrite()}><button class="btn btn--primary" disabled={busy() || !dirty()}>{t('domain-editor-preview')}</button></Show>
        </form>
        <For each={(['aliases', 'hosts'] as const)}>{kind => <section class={css.card} aria-label={t(`domain-editor-${kind}`)}><h4>{t(`domain-editor-${kind}`)}</h4><p>{t(`domain-editor-${kind}-help`)}</p><For each={d()[kind]}>{value => <div class={css.listRow}><span>{value}</span><Show when={canUpstream()}><button class="btn btn--ghost" disabled={busy()} onClick={() => void mapping(kind, value, true)} aria-label={t('domain-editor-remove-mapping', { value })}>{t('domain-editor-remove')}</button></Show></div>}</For><Show when={canUpstream()}><form onSubmit={e => { e.preventDefault(); void mapping(kind, kind === 'aliases' ? alias() : host()); }}><label class="field"><span>{t(`domain-editor-add-${kind}`)}</span><input required disabled={busy()} value={kind === 'aliases' ? alias() : host()} onInput={e => kind === 'aliases' ? setAlias(e.currentTarget.value) : setHost(e.currentTarget.value)} /></label><button class="btn" disabled={busy()}>{t('domain-editor-add')}</button></form></Show></section>}</For>
      </>}</Show>
    </Show>
    <Show when={preview()}>{p => <ConfirmDialog open title={t('domain-editor-review')} confirmLabel={t('domain-editor-apply')} danger={false} onCancel={() => setPreview(undefined)} onConfirm={async () => { const d = selected()!; try { await api.save(d.name, p().changes, d.etag); setPreview(undefined); await refresh(); setSuccess(t('domain-editor-saved')); } catch (e) { setPreview(undefined); setError(errorText(e)); } }}><p>{t('domain-editor-review-help', { name: p().result.after.name })}</p><ul><For each={Object.keys(p().changes)}>{key => <li><strong>{t(`domain-editor-change-${key}`)}</strong><p>{t('domain-editor-before')}: {summary(key, p().result.before)}</p><p>{t('domain-editor-after')}: {summary(key, p().result.after)}</p></li>}</For></ul></ConfirmDialog>}</Show>
    <Show when={confirmation()}>{kind => <ConfirmDialog open title={t(`domain-editor-confirm-${kind()}`, { name: selected()?.name ?? '' })} confirmLabel={kind() === 'delete' ? t('admin-delete') : t('domain-editor-continue')} {...(kind() === 'delete' ? { typedConfirmation: selected()?.name ?? '' } : {})} onCancel={() => setConfirmation(undefined)} onConfirm={async () => { const d = selected()!; if (kind() === 'delete') { await api.remove(d.name, d.etag, force()); setConfirmation(undefined); setSelected(undefined); setRows(await api.list()); } else if (kind() === 'disable') { await api.enable(d.name, false, d.etag); await refresh(); } else if (kind() === 'switch') await load(switchName()); else await refresh(); setConfirmation(undefined); }}><Show when={kind() === 'delete'}><p>{t('domain-editor-delete-help')}</p><label><input type="checkbox" checked={force()} onChange={e => setForce(e.currentTarget.checked)} /> {t('domain-editor-force')}</label></Show></ConfirmDialog>}</Show>
  </section>;
}
