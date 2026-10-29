import { ewsBackendProblem, ewsSelectionProblem } from './ewsBackendModel.ts';
import { EwsFields } from './EwsFields.tsx';
import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { domainBackends, domainsApi, type DomainBackend, type DomainDetail, type DomainEndpoint, type DomainsApi, type UsernameFormat } from '../../../api/admin/domains.ts';
import { loadCatalog, t } from '../../../i18n';
import { useDirtyGuard } from '../shell/useDirtyGuard.ts';
import * as css from '../../Settings/styles.css.ts';
import * as ux from '../shell/ux.css.ts';
import { TransportFields } from './TransportFields.tsx';
import { AuthFields } from './AuthFields.tsx';
import type { PasswordMechanism } from '../../../api/admin/domain-auth.ts';

const endpoint = (protocol: DomainBackend['protocol']): DomainEndpoint => ({ host: '', port: protocol === 'smtp' ? 465 : protocol === 'pop3' ? 995 : protocol === 'sieve' ? 4190 : 993, tls: protocol === 'sieve' ? 'starttls' : 'implicit', auth: 'password', allowPrivate: false });
export function BackendEditor(props: { domain: DomainDetail; backend?: DomainBackend | undefined; api: DomainsApi; advancedReady: boolean; authentication?: Record<string, PasswordMechanism[]> | undefined; onSaved: () => Promise<void>; onBack: () => void }): JSX.Element {
  const initial: DomainBackend = props.backend ?? { id: '', role: 'mail', protocol: props.domain.mode === 'proxy' ? 'jmap' : 'imap', enabled: true, preferred: false, priority: 100, ...(props.domain.mode === 'proxy' ? { sessionUrl: '' } : { endpoint: endpoint('imap') }), username: { format: 'fullEmail' } };
  const [value, setValue] = createSignal(structuredClone(initial));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [identity, setIdentity] = createSignal(`user@${props.domain.name}`);
  const [preview, setPreview] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);
  const dirty = () => !saved() && JSON.stringify(value()) !== JSON.stringify(initial);
  useDirtyGuard(dirty);
  onMount(() => void loadCatalog('admin-domain-backends'));
  const change = (patch: Partial<DomainBackend>) => { setValue(current => ({ ...current, ...patch })); setPreview(null); };
  const wire = () => value().endpoint ?? endpoint(value().protocol);
  const changeEndpoint = (patch: Partial<DomainEndpoint>) => change({ endpoint: { ...wire(), ...patch } });
  const username = (): UsernameFormat => { const current = value().username ?? { format: 'fullEmail' as const }; if (current.format === 'custom' && !current.realm) return { format: 'custom', template: current.template }; return current; };
  function protocol(next: DomainBackend['protocol']): void {
    const common = { ...value(), protocol: next, role: next === 'smtp' ? 'submission' as const : next === 'sieve' ? 'rules' as const : 'mail' as const };
    delete common.endpoint; delete common.sessionUrl; delete common.ews;
    if (next === 'jmap') { common.sessionUrl = ''; common.username = { format: 'fullEmail' }; } else if(next === 'ews') common.ews = {endpointUrl:'',authentication:'basic'}; else common.endpoint = endpoint(next);
    setValue(common); setPreview(null);
  }
  function format(next: UsernameFormat['format']): void {
    change({ username: next === 'domainUser' ? { format: next, domain: '' } : next === 'userRealm' ? { format: next, realm: '' } : next === 'custom' ? { format: next, template: '{email}' } : { format: next } });
  }
  async function save(event: Event): Promise<void> {
    event.preventDefault(); setBusy(true); setError(null);
    try {
      const edited = { ...value(), username: username() };
      const rows = domainBackends(props.domain.upstream).filter(row => row.id !== props.backend?.id).map(row => edited.preferred && row.role === edited.role ? { ...row, preferred: false } : row);
      rows.push(edited);
      const problem = (edited.protocol === 'ews' ? ewsBackendProblem(edited) : null) ?? ewsSelectionProblem(rows);
      if (problem) throw new Error(t(problem));
      await props.api.save(props.domain.name, { upstream: { version: 2, backends: rows, ...(props.domain.upstream.dav ? { dav: props.domain.upstream.dav } : {}), ...(props.domain.upstream.failover ? { failover: true } : {}) } }, props.domain.etag);
      setSaved(true); await props.onSaved();
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setBusy(false); }
  }
  return <section class={ux.page} aria-label={t('backend-editor-title')}>
    <header class={ux.pageHead}><div><h2 class={ux.pageTitle}>{t(props.backend ? 'backend-editor-edit' : 'backend-editor-add')}</h2><p class={ux.pageDescription}>{props.domain.name} · {t('backend-editor-intro')}</p></div><div class={css.actions}><button class={css.button} type="button" onClick={props.onBack}>{t('backend-editor-back')}</button></div></header>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <form class={css.stack} onSubmit={event => void save(event)}>
      <fieldset class={css.section} disabled={busy()}><legend class={css.heading}>{t('backend-editor-identity')}</legend>
        <label class={css.field}>{t('backend-editor-id')}<input class={css.input} required pattern="[A-Za-z0-9_-]{1,64}" maxlength="64" disabled={!!props.backend} value={value().id} onInput={event => change({ id: event.currentTarget.value })} title={t('backend-editor-id-help')} /><small>{t('backend-editor-id-help')}</small></label>
        <label class={css.field}>{t('backend-editor-protocol')}<select class={css.select} value={value().protocol} onChange={event => protocol(event.currentTarget.value as DomainBackend['protocol'])} title={t('backend-editor-protocol-help')}><For each={['imap', 'pop3', 'jmap', 'smtp', 'sieve', 'ews'] as const}>{option => <option value={option} disabled={props.domain.mode === 'proxy' ? option !== 'jmap' : option === 'jmap'}>{option.toUpperCase()}</option>}</For></select><small>{t('backend-editor-protocol-help')}</small></label>
        <label class={css.field}>{t('backend-editor-priority')}<input class={css.input} type="number" required min="0" max="65535" value={value().priority ?? 100} onInput={event => change({ priority: event.currentTarget.valueAsNumber })} title={t('backend-editor-priority-help')} /><small>{t('backend-editor-priority-help')}</small></label>
        <label class={css.field}><input type="checkbox" checked={value().enabled !== false} onChange={event => change({ enabled: event.currentTarget.checked, ...(!event.currentTarget.checked ? { preferred: false } : {}) })} title={t('backend-editor-enabled-help')} /><span>{t('backend-editor-enabled')}</span></label><p class={css.meta}>{t('backend-editor-enabled-help')}</p>
        <label class={css.field}><input type="checkbox" disabled={value().enabled === false} checked={value().preferred ?? false} onChange={event => change({ preferred: event.currentTarget.checked })} title={t('backend-editor-preferred-help')} /><span>{t('backend-editor-preferred')}</span></label><p class={css.meta}>{t('backend-editor-preferred-help')}</p>
      </fieldset>
      <fieldset class={css.section} disabled={busy()}><legend class={css.heading}>{t('backend-editor-connection')}</legend>
        <Show when={value().protocol === 'ews'} fallback={<Show when={value().protocol === 'jmap'} fallback={<>
          <label class={css.field}>{t('backend-editor-host')}<input class={css.input} required value={wire().host} onInput={event => changeEndpoint({ host: event.currentTarget.value })} title={t('backend-editor-host-help')} /><small>{t('backend-editor-host-help')}</small></label>
          <label class={css.field}>{t('backend-editor-port')}<input class={css.input} type="number" required min="1" max="65535" value={wire().port} onInput={event => changeEndpoint({ port: event.currentTarget.valueAsNumber })} title={t('backend-editor-port-help')} /><small>{t('backend-editor-port-help')}</small></label>
          <label class={css.field}>{t('backend-editor-security')}<select class={css.select} value={wire().tls} onChange={event => changeEndpoint({ tls: event.currentTarget.value as DomainEndpoint['tls'] })} title={t('backend-editor-security-help')}><option value="implicit">TLS</option><option value="starttls">STARTTLS</option><option value="none">{t('backend-editor-plaintext')}</option></select><small>{t('backend-editor-security-help')}</small></label>
          <Show when={wire().tls === 'none'}><p role="alert">{t('backend-editor-plaintext-warning')}</p></Show>
          <label class={css.field}><input type="checkbox" checked={wire().allowPrivate ?? false} onChange={event => changeEndpoint({ allowPrivate: event.currentTarget.checked })} title={t('backend-editor-private-help')} /><span>{t('backend-editor-private')}</span></label><p class={css.meta}>{t('backend-editor-private-help')}</p>
          <TransportFields value={wire()} ready={props.advancedReady} update={next => change({ endpoint: next })} />
        </>}><label class={css.field}>{t('backend-editor-session-url')}<input class={css.input} type="url" required value={value().sessionUrl ?? ''} onInput={event => change({ sessionUrl: event.currentTarget.value })} title={t('backend-editor-session-help')} /><small>{t('backend-editor-session-help')}</small></label></Show>}><EwsFields value={value().ews!} change={ews => change({ews})} /></Show>
      </fieldset>
      <fieldset class={css.section} disabled={busy()}><legend class={css.heading}>{t('backend-editor-auth')}</legend><p class={css.meta}>{t('backend-editor-auth-help')}</p>
        <Show when={value().protocol !== 'jmap' && value().protocol !== 'ews'}><AuthFields value={wire().authentication} supported={props.authentication?.[value().protocol]} change={authentication => { const next = { ...wire() }; if (authentication) next.authentication = authentication; else delete next.authentication; change({ endpoint: next }); }} /></Show>
        <label class={css.field}>{t('backend-editor-username-format')}<select class={css.select} value={username().format} disabled={value().protocol === 'jmap'} onChange={event => format(event.currentTarget.value as UsernameFormat['format'])} title={t('backend-editor-username-help')}><For each={['fullEmail', 'localUser', 'domainUser', 'userRealm', 'custom'] as const}>{option => <option value={option}>{t(`backend-editor-format-${option}`)}</option>}</For></select><small>{t('backend-editor-username-help')}</small></label>
        <Show when={username().format === 'domainUser'}><label class={css.field}>{t('backend-editor-auth-domain')}<input class={css.input} required value={(username() as { domain: string }).domain} onInput={event => change({ username: { format: 'domainUser', domain: event.currentTarget.value } })} title={t('backend-editor-auth-domain-help')} /><small>{t('backend-editor-auth-domain-help')}</small></label></Show>
        <Show when={username().format === 'userRealm' || username().format === 'custom'}><label class={css.field}>{t('backend-editor-realm')}<input class={css.input} required={username().format === 'userRealm'} value={(username() as { realm?: string }).realm ?? ''} onInput={event => change({ username: { ...username(), realm: event.currentTarget.value } as UsernameFormat })} title={t('backend-editor-realm-help')} /><small>{t('backend-editor-realm-help')}</small></label></Show>
        <Show when={username().format === 'custom'}><label class={css.field}>{t('backend-editor-template')}<input class={css.input} required maxlength="512" value={(username() as { template: string }).template} onInput={event => change({ username: { ...username(), template: event.currentTarget.value } as UsernameFormat })} title={t('backend-editor-template-help')} /><small>{t('backend-editor-template-help')}</small></label></Show>
        <label class={css.field}>{t('backend-editor-example')}<input class={css.input} required type="email" value={identity()} onInput={event => { setIdentity(event.currentTarget.value); setPreview(null); }} title={t('backend-editor-example-help')} /><small>{t('backend-editor-example-help')}</small></label>
        <div class={css.actions}><button class={css.button} type="button" onClick={() => { setBusy(true); setError(null); void (props.api.usernamePreview ?? domainsApi.usernamePreview)(identity(), username()).then(result => setPreview(result.authenticationUsername)).catch(cause => setError(String(cause))).finally(() => setBusy(false)); }}>{t('backend-editor-preview-username')}</button></div>
        <Show when={preview()}><p role="status">{t('backend-editor-preview-result')}: <code>{preview()}</code></p></Show>
      </fieldset>
      <p class={css.meta}>{t('backend-editor-save-help')}</p><div class={css.actions}><button class={css.button} type="submit" disabled={busy()}>{t('backend-editor-save')}</button></div>
    </form>
  </section>;
}
