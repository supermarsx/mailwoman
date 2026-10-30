// Security > Country database (t96): the local database country policies read.
// Status, provider and licence key, downloads, upload, rollback and a test lookup.
// Every refresh keeps the page mounted (t75): state lives in signals, and the
// last good status stays on screen while an action runs.
import { createSignal, For, onMount, Show, useContext, type JSX } from 'solid-js';
import { geoipApi, type GeoipApi, type GeoipConfig, type GeoipLookup, type GeoipProvider, type GeoipStatus } from '../../api/admin/geoip.ts';
import { formatBytes, formatDateTime, formatNumber, loadCatalog, t } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { DataTable } from './shell/DataTable.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { useDirtyGuard } from './shell/useDirtyGuard.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';

const INTERVALS = [1, 7, 14, 30] as const;
type Draft = { provider: GeoipConfig['provider']; customUrl: string; customChecksumUrl: string; customRedirectHosts: string; autoUpdate: boolean; intervalDays: number };
const draftOf = (c: GeoipConfig): Draft => ({ provider: c.provider, customUrl: c.customUrl, customChecksumUrl: c.customChecksumUrl, customRedirectHosts: c.customRedirectHosts.join('\n'), autoUpdate: c.autoUpdate, intervalDays: c.intervalDays });
const when = (seconds: number | null | undefined): string => (seconds ? formatDateTime(seconds * 1000) : t('admin-geoip-never'));

export function CountryDatabase(props: { api?: GeoipApi }): JSX.Element {
  const api = (): GeoipApi => props.api ?? geoipApi;
  const shell = useContext(ShellContext);
  const [status, setStatus] = createSignal<GeoipStatus>();
  const [etag, setEtag] = createSignal('');
  const [draft, setDraft] = createSignal<Draft>();
  const [baseline, setBaseline] = createSignal('');
  const [licenceKey, setLicenceKey] = createSignal('');
  const [clearKey, setClearKey] = createSignal(false);
  const [caPem, setCaPem] = createSignal<string>();
  const [busy, setBusy] = createSignal<string>('');
  const [error, setError] = createSignal('');
  const [notice, setNotice] = createSignal('');
  const [file, setFile] = createSignal<File>();
  const [ip, setIp] = createSignal('');
  const [lookup, setLookup] = createSignal<GeoipLookup>();
  const [lookupError, setLookupError] = createSignal('');
  let fileInput: HTMLInputElement | undefined;

  const writable = (): boolean => (!shell || shell.can.can('security.write')) && status()?.writable === true;
  const dirty = (): boolean => !!draft() && (JSON.stringify(draft()) !== baseline() || licenceKey() !== '' || clearKey() || caPem() !== undefined);
  useDirtyGuard(dirty);

  function accept(next: GeoipStatus, tag?: string): void {
    setStatus(next);
    if (tag !== undefined) setEtag(tag);
    // Keep an unsaved edit of the source form; refresh it only when it is clean.
    if (!dirty()) { const d = draftOf(next.config); setDraft(d); setBaseline(JSON.stringify(d)); }
  }
  async function reload(): Promise<void> {
    try { const r = await api().read(); setEtag(r.etag); setStatus(r.data); const d = draftOf(r.data.config); setDraft(d); setBaseline(JSON.stringify(d)); setError(''); }
    catch (cause) { setError(describeAdminError(cause)); }
  }
  async function act(name: string, run: () => Promise<GeoipStatus>, success: (s: GeoipStatus) => string): Promise<void> {
    if (busy()) return;
    setBusy(name); setError(''); setNotice('');
    try { const next = await run(); accept(next); setNotice(success(next)); const r = await api().read(); setEtag(r.etag); accept(r.data); }
    catch (cause) { setError(describeAdminError(cause)); }
    finally { setBusy(''); }
  }
  const outcomeText = (s: GeoipStatus): string => s.outcome?.result === 'up-to-date' ? t('admin-geoip-up-to-date') : t('admin-geoip-installed');
  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const d = draft(); if (!d || busy()) return;
    setBusy('save'); setError(''); setNotice('');
    try {
      const pem = caPem();
      const r = await api().save({
        provider: d.provider, customUrl: d.customUrl.trim(), customChecksumUrl: d.customChecksumUrl.trim(),
        customRedirectHosts: d.customRedirectHosts.split(/[\s,]+/).map(h => h.trim()).filter(Boolean),
        autoUpdate: d.autoUpdate, intervalDays: d.intervalDays,
        ...(licenceKey() ? { licenceKey: licenceKey() } : {}), ...(clearKey() ? { clearLicenceKey: true } : {}),
        ...(pem !== undefined ? { customCaPem: pem } : {}),
      }, etag());
      setLicenceKey(''); setClearKey(false); setCaPem(undefined);
      setEtag(r.etag); setStatus(r.data); const nd = draftOf(r.data.config); setDraft(nd); setBaseline(JSON.stringify(nd));
      setNotice(t('admin-geoip-saved'));
    } catch (cause) { setError(describeAdminError(cause)); }
    finally { setBusy(''); }
  }
  async function upload(): Promise<void> {
    const f = file(); if (!f) return;
    await act('upload', () => api().upload(f), () => t('admin-geoip-uploaded'));
    setFile(undefined); if (fileInput) fileInput.value = '';
  }
  async function test(event: SubmitEvent): Promise<void> {
    event.preventDefault(); setLookupError('');
    try { setLookup(await api().lookup(ip())); } catch (cause) { setLookup(undefined); setLookupError(describeAdminError(cause)); }
  }
  const change = (patch: Partial<Draft>): void => { setNotice(''); setDraft(d => d && ({ ...d, ...patch })); };
  const providerOf = (id: string | undefined): GeoipProvider | undefined => status()?.providers.find(p => p.id === id);
  const stateLabel = (s: GeoipStatus): string => t(`admin-geoip-state-${s.state}`);
  const sourceLabel = (source: string): string => providerOf(source)?.name ?? t(`admin-geoip-source-${['upload', 'seed', 'environment'].includes(source) ? source : 'unknown'}`);
  const rows = (): { id: string; value: string }[] => {
    const s = status(); if (!s) return [];
    const db = s.database;
    return [
      { id: 'state', value: stateLabel(s) },
      { id: 'managed', value: t(s.managedBy === 'environment' ? 'admin-geoip-managed-environment' : 'admin-geoip-managed-console') },
      ...(db ? [
        { id: 'source', value: sourceLabel(db.source) + (db.release ? ` (${db.release})` : '') },
        { id: 'type', value: db.databaseType || '—' },
        { id: 'built', value: db.buildEpoch ? formatDateTime(db.buildEpoch * 1000) : '—' },
        { id: 'size', value: formatBytes(db.sizeBytes) },
        { id: 'networks', value: db.networks === null ? '—' : formatNumber(db.networks) },
        { id: 'verified', value: t(db.checksumVerified ? 'admin-geoip-verified-checksum' : 'admin-geoip-verified-structure') },
        ...(db.installedAt ? [{ id: 'installed', value: `${when(db.installedAt)} · ${db.installedBy ?? ''}` }] : []),
      ] : []),
      ...(s.managedBy === 'console' ? [
        { id: 'last-check', value: when(s.schedule.lastCheckAt) },
        { id: 'next-check', value: s.schedule.autoUpdate ? (s.schedule.waitingForFirstDownload ? t('admin-geoip-next-after-download') : when(s.schedule.nextCheckAt)) : t('admin-geoip-auto-off') },
        ...(s.schedule.lastError ? [{ id: 'last-error', value: s.schedule.lastError }] : []),
        { id: 'directory', value: s.dataDirectory ?? t('admin-geoip-no-directory') },
      ] : []),
    ];
  };

  onMount(() => { void loadCatalog('admin-geoip'); void reload(); });

  return <section class={css.section} aria-labelledby="admin-geoip-title">
    <div class={ux.pageHead}>
      <h2 id="admin-geoip-title" class={css.heading}>{t('admin-geoip-title')}</h2>
      <div class={ux.pageActions}>
        <button type="button" class="btn btn--ghost" onClick={() => void shell?.router.navigate(sectionPath('security', ['geography']))}>{t('admin-obs-geo-title')}</button>
        <button type="button" class="btn" disabled={!!busy()} onClick={() => void reload()}>{t('admin-geoip-refresh')}</button>
      </div>
    </div>
    <p class={css.note}>{t('admin-geoip-intro')}</p>
    <Show when={error()}><Feedback kind="error" message={error()} /></Show>
    <Show when={notice()}><p role="status">{notice()}</p></Show>
    <Show when={status()} fallback={<Show when={!error()}><p role="status">{t('admin-geoip-loading')}</p></Show>}>{s => <>
      <Show when={s().state !== 'loaded' && s().policyActive}><Feedback kind="error" message={t('admin-geoip-refusing')} /></Show>
      <Show when={s().state !== 'loaded' && !s().policyActive}><Feedback kind="info" message={t('admin-geoip-missing-inactive')} /></Show>
      <Show when={s().state === 'invalid' && s().error}><p class={css.note}>{t('admin-geoip-invalid-reason', { reason: s().error ?? '' })}</p></Show>
      <Show when={s().database?.stale}><Feedback kind="info" message={t('admin-geoip-stale', { days: s().staleAfterDays })} /></Show>
      <Show when={s().managedBy === 'environment'}><Feedback kind="info" message={t('admin-geoip-environment')} /></Show>
      <Show when={s().managedBy === 'console' && !s().dataDirectory}><Feedback kind="info" message={t('admin-geoip-no-directory-help')} /></Show>

      <div class={css.card}>
        <DataTable caption={t('admin-geoip-status')} rows={rows()} rowKey={row => row.id} state="ready" emptyText=""
          columns={[{ id: 'field', header: t('admin-geoip-field'), cell: row => t(`admin-geoip-field-${row.id}`) }, { id: 'value', header: t('admin-geoip-value'), cell: row => <span class={row.id === 'directory' ? css.mono : undefined}>{row.value}</span> }]} />
      </div>

      <Show when={s().database && providerOf(s().database?.source)?.attribution}>
        <p class={css.note} data-testid="geoip-attribution">
          {t('admin-geoip-attribution-active')} <a href={providerOf(s().database?.source)?.attributionUrl ?? '#'} target="_blank" rel="noopener noreferrer">{providerOf(s().database?.source)?.attribution}</a>
          {' · '}<a href={providerOf(s().database?.source)?.licenceUrl ?? '#'} target="_blank" rel="noopener noreferrer">{providerOf(s().database?.source)?.licence}</a>
        </p>
      </Show>

      <Show when={s().managedBy === 'console'}>
        <div class={ux.pageActions} role="group" aria-label={t('admin-geoip-actions')}>
          <button type="button" class="btn btn--primary" disabled={!writable() || !!busy()} onClick={() => void act('download', () => api().download(), outcomeText)}>{busy() === 'download' ? t('admin-geoip-downloading') : t('admin-geoip-download')}</button>
          <button type="button" class="btn" disabled={!writable() || !!busy()} onClick={() => void act('check', () => api().check(), outcomeText)}>{busy() === 'check' ? t('admin-geoip-checking') : t('admin-geoip-check')}</button>
          <button type="button" class="btn" disabled={!writable() || !!busy() || !s().previous} onClick={() => void act('rollback', () => api().rollback(), () => t('admin-geoip-rolled-back'))}>{t('admin-geoip-rollback')}</button>
        </div>
        <Show when={s().previous}>{p => <p class={css.note}>{t('admin-geoip-previous', { source: sourceLabel(p().source), built: p().buildEpoch ? formatDateTime(p().buildEpoch! * 1000) : '—' })}</p>}</Show>

        <fieldset class={ux.fieldset}>
          <legend class={ux.legend}>{t('admin-geoip-upload-title')}</legend>
          <p class={ux.fieldsetHelp}>{t('admin-geoip-upload-help')}</p>
          <div class={ux.pageActions}>
            <label class="field"><span>{t('admin-geoip-upload-file')}</span>
              <input ref={el => { fileInput = el; }} type="file" accept=".mmdb,.gz,application/octet-stream" disabled={!writable() || !!busy()} onChange={e => setFile(e.currentTarget.files?.[0])} /></label>
            <button type="button" class="btn" disabled={!writable() || !!busy() || !file()} onClick={() => void upload()}>{busy() === 'upload' ? t('admin-geoip-uploading') : t('admin-geoip-upload')}</button>
          </div>
        </fieldset>

        <Show when={draft()}>{d => <form class={ux.form} onSubmit={e => void save(e)} aria-labelledby="admin-geoip-source">
          <fieldset class={ux.fieldset}>
            <legend id="admin-geoip-source" class={ux.legend}>{t('admin-geoip-source-title')}</legend>
            <p class={ux.fieldsetHelp}>{t('admin-geoip-source-help')}</p>
            <label class="field"><span>{t('admin-geoip-provider')}</span>
              <select value={d().provider} disabled={!writable() || !!busy()} onChange={e => { setLicenceKey(''); setClearKey(false); change({ provider: e.currentTarget.value as Draft['provider'] }); }}>
                <For each={s().providers}>{p => <option value={p.id}>{p.name}</option>}</For>
              </select></label>
            <Show when={providerOf(d().provider)}>{p => <p class={ux.fieldsetHelp} data-testid="geoip-provider-licence">
              <Show when={p().licence} fallback={t('admin-geoip-custom-licence')}>
                {t('admin-geoip-licence', { licence: p().licence ?? '' })}{' '}<a href={p().licenceUrl ?? '#'} target="_blank" rel="noopener noreferrer">{t('admin-geoip-licence-terms')}</a>
                {' · '}{t('admin-geoip-attribution-required')} <a href={p().attributionUrl ?? '#'} target="_blank" rel="noopener noreferrer">{p().attribution}</a>
              </Show>
            </p>}</Show>
            <Show when={providerOf(d().provider)?.needsKey}>
              <label class="field"><span>{t(d().provider === 'ipinfo' ? 'admin-geoip-token' : 'admin-geoip-licence-key')}</span>
                <input type="password" autocomplete="new-password" maxlength={128} value={licenceKey()} disabled={!writable() || !!busy() || clearKey()} onInput={e => setLicenceKey(e.currentTarget.value)} />
                <small>{t(s().config.licenceKeyConfigured && s().config.provider === d().provider ? 'admin-geoip-key-stored' : 'admin-geoip-key-needed')}</small></label>
              <Show when={s().config.licenceKeyConfigured && s().config.provider === d().provider}>
                <label class={ux.checkboxRow}><input type="checkbox" checked={clearKey()} disabled={!writable() || !!busy()} onChange={e => { setClearKey(e.currentTarget.checked); setLicenceKey(''); }} /> {t('admin-geoip-key-clear')}</label>
              </Show>
            </Show>
            <Show when={d().provider === 'custom'}>
              <label class="field"><span>{t('admin-geoip-custom-url')}</span><input type="url" inputmode="url" maxlength={2048} required value={d().customUrl} disabled={!writable() || !!busy()} onInput={e => change({ customUrl: e.currentTarget.value })} /><small>{t('admin-geoip-custom-url-help')}</small></label>
              <label class="field"><span>{t('admin-geoip-custom-checksum')}</span><input type="url" inputmode="url" maxlength={2048} value={d().customChecksumUrl} disabled={!writable() || !!busy()} onInput={e => change({ customChecksumUrl: e.currentTarget.value })} /><small>{t('admin-geoip-custom-checksum-help')}</small></label>
              <label class="field"><span>{t('admin-geoip-custom-hosts')}</span><textarea rows={2} value={d().customRedirectHosts} disabled={!writable() || !!busy()} onInput={e => change({ customRedirectHosts: e.currentTarget.value })} /><small>{t('admin-geoip-custom-hosts-help')}</small></label>
              <label class="field"><span>{t('admin-geoip-custom-ca')}</span><textarea rows={3} class={css.mono} value={caPem() ?? ''} placeholder={s().config.customCaConfigured ? t('admin-geoip-custom-ca-stored') : '-----BEGIN CERTIFICATE-----'} disabled={!writable() || !!busy()} onInput={e => setCaPem(e.currentTarget.value)} /><small>{t('admin-geoip-custom-ca-help')}</small></label>
            </Show>
            <label class={ux.checkboxRow}><input type="checkbox" checked={d().autoUpdate} disabled={!writable() || !!busy()} onChange={e => change({ autoUpdate: e.currentTarget.checked })} /> {t('admin-geoip-auto')}</label>
            <label class="field"><span>{t('admin-geoip-interval')}</span>
              <select value={String(d().intervalDays)} disabled={!writable() || !!busy() || !d().autoUpdate} onChange={e => change({ intervalDays: Number(e.currentTarget.value) })}>
                <For each={[...new Set([...INTERVALS, d().intervalDays])].sort((a, b) => a - b)}>{n => <option value={String(n)}>{t('admin-geoip-interval-days', { count: n })}</option>}</For>
              </select><small>{t('admin-geoip-interval-help')}</small></label>
            <p class={ux.fieldsetHelp}>{t('admin-geoip-step-up')}</p>
            <div><button type="submit" class="btn btn--primary" disabled={!writable() || !!busy() || !dirty()}>{busy() === 'save' ? t('admin-geoip-saving') : t('admin-geoip-save')}</button></div>
          </fieldset>
        </form>}</Show>
      </Show>

      <form class={ux.fieldset} onSubmit={e => void test(e)} aria-labelledby="admin-geoip-lookup">
        <h3 id="admin-geoip-lookup" class={ux.legend}>{t('admin-geoip-lookup-title')}</h3>
        <p class={ux.fieldsetHelp}>{t('admin-geoip-lookup-help')}</p>
        <div class={ux.pageActions}>
          <label class="field"><span>{t('admin-geoip-lookup-ip')}</span><input inputmode="text" autocomplete="off" spellcheck={false} maxlength={64} required value={ip()} onInput={e => setIp(e.currentTarget.value)} /></label>
          <button type="submit" class="btn" disabled={!ip().trim()}>{t('admin-geoip-lookup')}</button>
        </div>
        <Show when={lookupError()}><p role="alert">{lookupError()}</p></Show>
        <Show when={lookup()}>{r => <p role="status" data-testid="geoip-lookup-result">{
          r().database === 'unavailable' ? t('admin-geoip-lookup-unavailable') : r().country ? t('admin-geoip-lookup-country', { ip: r().ip, country: r().country ?? '' }) : t('admin-geoip-lookup-unknown', { ip: r().ip })
        }</p>}</Show>
      </form>
    </>}</Show>
  </section>;
}
