import { createSignal, For, Index, onMount, Show, type JSX } from 'solid-js';
import { directoryApi, type DirectoryConfiguration, type DirectoryEntry, type DirectoryWrite } from '../../api/admin/directory.ts';
import { loadCatalog, t } from '../../i18n';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
const secretValue = (row: DirectoryWrite): string => row.password.action === 'set' ? row.password.value : '';
const draft = (entry: DirectoryEntry): DirectoryWrite => {
  const { revision: _revision, password: _password, ...fields } = entry;
  return { ...fields, attrMap: { ...fields.attrMap }, password: { action: 'keep' } };
};
export function DirectoryIntegration(props: { canWrite: boolean; service?: typeof directoryApi }): JSX.Element {
  const service = props.service ?? directoryApi;
  const [config, setConfig] = createSignal<DirectoryConfiguration>();
  const [entries, setEntries] = createSignal<DirectoryWrite[]>([]);
  const [busy, setBusy] = createSignal(false); const [error, setError] = createSignal(''); const [saved, setSaved] = createSignal(false);
  async function load(): Promise<void> { setBusy(true); setError(''); try { const value = await service.load(); setConfig(value); setEntries(value.entries.map(draft)); } catch (e) { setError(describeAdminError(e)); } finally { setBusy(false); } }
  onMount(() => { void loadCatalog('admin-directory'); void load(); });
  function change(index: number, update: Partial<DirectoryWrite>): void { setEntries(value => value.map((row, i) => i === index ? { ...row, ...update } : row)); setSaved(false); }
  async function save(event: SubmitEvent): Promise<void> { event.preventDefault(); const current = config(); if (!current || !props.canWrite) return; setBusy(true); setError(''); setSaved(false); try { await service.save(entries(), current.etag); await load(); setSaved(true); } catch (e) { setError(describeAdminError(e)); } finally { setBusy(false); } }
  const pinned = (row: DirectoryWrite): boolean => Boolean(config()?.environment.pinned && row.bindDn === config()?.environment.bindDn);
  return <section aria-label={t('directory-title')} class={css.section}>
    <h3>{t('directory-title')}</h3><p>{t('directory-help')}</p>
    <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
    <Feedback kind="success" as="toast" message={saved()?t('directory-saved'):null}/>
    <button type="button" class="btn" disabled={busy()} onClick={() => void load()}>{t('directory-reload')}</button>
    <Show when={config()}>{current => <>
      <Show when={current().restartRequired}><p role="status">{t('directory-restart')}</p></Show>
      <Show when={current().transportStatus === 'proxyUnsupported'}><p role="alert">{t('directory-proxy-blocked')}</p></Show>
      <Show when={current().environment.pinned}><p>{t('directory-env-pin')} <code>{current().environment.bindDn}</code></p></Show>
      <form onSubmit={e => void save(e)} aria-label={t('directory-title')}>
        <Index each={entries()}>{(row, index) => <fieldset class={css.card} disabled={!props.canWrite || busy()}>
          <legend>{row().id}</legend>
          <label class="field">{t('directory-url')}<input type="url" required value={row().url} disabled={pinned(row())} onInput={e => change(index, { url: e.currentTarget.value })} /></label>
          <label class="field">{t('directory-base')}<input value={row().baseDn} onInput={e => change(index, { baseDn: e.currentTarget.value })} /></label>
          <label class="field">{t('directory-bind')}<input value={row().bindDn ?? ''} disabled={pinned(row())} onInput={e => change(index, { bindDn: e.currentTarget.value || null })} /></label>
          <label class="field">{t('directory-tls')}<select value={row().tls} onChange={e => change(index, { tls: e.currentTarget.value as DirectoryWrite['tls'] })}><option value="ldaps">LDAPS</option><option value="starttls">StartTLS</option><option value="none">{t('directory-plaintext')}</option></select></label>
          <label class="field">{t('directory-priority')}<input type="number" min={-2147483648} max={2147483647} value={row().priority} onInput={e => change(index, { priority: Number(e.currentTarget.value) })} /></label>
          <label class="field">{t('directory-connection')}<select value={row().connectionMode} onChange={e => change(index, { connectionMode: e.currentTarget.value as DirectoryWrite['connectionMode'] })}><option value="blocked">{t('directory-blocked')}</option><option value="direct">{t('directory-direct')}</option></select></label>
          <label class="field"><input type="checkbox" checked={row().enabled} onChange={e => change(index, { enabled: e.currentTarget.checked })} />{t('directory-enabled')}</label>
          <label class="field"><input type="checkbox" checked={row().allowPrivate} onChange={e => change(index, { allowPrivate: e.currentTarget.checked })} />{t('directory-private')}</label>
          <Show when={row().connectionMode === 'blocked'}><p>{t('directory-consent')}</p></Show>
          <p>{config()?.entries.find(e => e.id === row().id)?.password.set ? t('directory-password-set') : t('directory-password-unset')}</p>
          <label class="field">{t('directory-secret-action')}<select disabled={pinned(row())} value={row().password.action} onChange={e => change(index, { password: e.currentTarget.value === 'set' ? { action: 'set', value: '' } : { action: e.currentTarget.value as 'keep' | 'clear' } })}><option value="keep">{t('directory-keep')}</option><option value="set">{t('directory-replace')}</option><option value="clear">{t('directory-clear')}</option></select></label>
          <Show when={row().password.action === 'set'}><label class="field">{t('directory-password')}<input type="password" autocomplete="new-password" required maxLength={4096} value={secretValue(row())} onInput={e => change(index, { password: { action: 'set', value: e.currentTarget.value } })} /></label></Show>
          <For each={['display_name', 'mail', 'member', 'user_cert', 'photo'] as const}>{key => <label class="field">{t(`directory-attr-${key.replaceAll('_', '-')}`)}<input value={row().attrMap[key] ?? ''} maxLength={128} onInput={e => change(index, { attrMap: { ...row().attrMap, [key]: e.currentTarget.value || null } })} /></label>}</For>
          <button class="btn" type="button" onClick={() => setEntries(value => value.filter((_, i) => i !== index))}>{t('directory-remove')}</button>
        </fieldset>}</Index>
        <Show when={props.canWrite}><div class={css.grid}>
          <button type="button" class="btn" disabled={busy() || entries().length >= 32} onClick={() => setEntries(value => [...value, { id: crypto.randomUUID(), url: '', baseDn: '', bindDn: null, tls: 'ldaps', priority: 0, attrMap: {}, enabled: false, connectionMode: 'blocked', allowPrivate: false, password: { action: 'keep' } }])}>{t('directory-add')}</button>
          <button class="btn" type="submit" disabled={busy()}>{t('directory-save')}</button>
        </div></Show>
      </form>
    </>}</Show>
  </section>;
}
