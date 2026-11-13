import {createResource,createSignal,For,Show,onCleanup,onMount,type JSX} from 'solid-js';
import {t,isolate,loadCatalog} from '../../i18n';
import {formatDateTime} from '../../i18n/format.ts';
import {ApiKeyService,type Fetcher} from './service.ts';
import {ScopeBuilder,summarizeScope} from './ScopeBuilder.tsx';
import {readOnlyScope,type ApiKeyRecord,type ApiKeyScope,type MintedKey} from './types.ts';
import {toast} from '../../components/toast/index.ts';
import {Icon} from '../../components/Icon.tsx';
import {useEntityNavigation,useEditorGuard} from '../../screens/Settings/entityEditor.tsx';
import {PersonalAccountBoundary} from '../../screens/Settings/PersonalAccountBoundary.tsx';
import * as linked from '../../screens/Settings/linkedAccounts.css.ts';
import * as entity from '../../screens/Settings/entityTable.css.ts';
import * as layout from '../../screens/Settings/editor.css.ts';
import * as table from '../../screens/Settings/managementTable.css.ts';
import * as ux from '../../screens/Admin/shell/ux.css.ts';
import * as css from './styles.css.ts';

export interface ApiKeysProps {
  accountId:string;
  fetcher?:Fetcher;
  /** Tests inject an initial list; production fetches. */
  initialKeys?:ApiKeyRecord[];
}
export function ApiKeys(props:ApiKeysProps):JSX.Element {
  return <PersonalAccountBoundary>{()=> <OwnedApiKeys {...props}/>}</PersonalAccountBoundary>;
}
function OwnedApiKeys(props:ApiKeysProps):JSX.Element {
  onMount(()=>{void loadCatalog('apikeys');void loadCatalog('settings');});
  const service=new ApiKeyService(props.fetcher),navigation=useEntityNavigation('access');
  const [keys,{refetch}]=createResource<ApiKeyRecord[]>(()=>props.initialKeys??service.list());
  const [query,setQuery]=createSignal(''),[revoking,setRevoking]=createSignal<string>();
  let disposed=false;onCleanup(()=>{disposed=true;});
  const editing=()=>navigation.route().section==='access'&&navigation.route().editor?.kind==='new';
  const shown=()=>(keys()??[]).filter(key=>`${key.label} ${key.prefix} ${summarizeScope(key.scope).join(' ')}`.toLocaleLowerCase().includes(query().trim().toLocaleLowerCase()));
  async function revoke(prefix:string):Promise<void>{
    if(revoking())return;setRevoking(prefix);
    try{await service.revoke(prefix);if(!disposed)await refetch();}
    catch(cause){if(!disposed)toast.error(cause instanceof Error?cause.message:t('apikeys-error-revoke'));}
    finally{if(!disposed)setRevoking(undefined);}
  }
  return <div class={css.managementPanel} aria-label={t('apikeys-panel-label')}>
    <Show when={editing()} fallback={<section class={css.section} aria-label={t('apikeys-existing-label')}>
      <div class={linked.header}><div class={linked.headerText}><h2 class={css.heading}>{t('apikeys-heading')}</h2><p class={css.prose}>{t('apikeys-intro')}</p></div><button type="button" class={`${css.button} ${linked.addButton}`} onClick={()=>void navigation.navigate({section:'access',editor:{kind:'new'}})}><Icon name="plus"/>{t('apikeys-create')}</button></div>
      <Show when={keys.error}><p class={css.error} role="alert">{t('common-error-retry-help')}</p><button type="button" class={css.ghost} onClick={()=>void refetch()}>{t('common-retry')}</button></Show>
      <Show when={keys.loading&&!keys()}><p role="status">{t('common-loading')}</p></Show>
      <Show when={!keys.error&&!!keys()}>
        <div class={table.bar}><label class={table.search}>{t('common-search')}<input type="search" class={css.input} value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label><span class={css.meta}>{shown().length} / {keys()!.length}</span></div>
        <ul class={linked.table}><For each={shown()} fallback={<li class={linked.empty}>{t('apikeys-none')}</li>}>{key=><li class={entity.row}>
          <span class={entity.avatar} aria-hidden="true"><Icon name="key"/></span>
          <div class={linked.who}><div class={entity.name}><strong class={entity.label}>{isolate(key.label)}</strong><span class={key.revokedAt?css.statusRevoked:css.statusActive}>{t(key.revokedAt?'apikeys-status-revoked':'apikeys-status-active')}</span></div>
            <code class={entity.detail}>mwk_{key.prefix}</code><span class={entity.detail}>{summarizeScope(key.scope).join(' · ')}</span>
            <span class={entity.detail}>{t('apikeys-created',{date:isolate(formatDateTime(key.createdAt))})} · {key.lastUsedAt?t('apikeys-last-used',{date:isolate(formatDateTime(key.lastUsedAt))}):t('apikeys-never-used')}<Show when={key.revokedAt}>{date=>` · ${t('apikeys-revoked-at',{date:isolate(formatDateTime(date()))})}`}</Show></span>
          </div>
          <div class={linked.actions}><Show when={!key.revokedAt}><button type="button" class={`btn btn--danger ${ux.iconAction}`} disabled={!!revoking()} aria-label={`${t('apikeys-revoke')}: ${key.label}`} title={`${t('apikeys-revoke')}: ${key.label}`} onClick={()=>void revoke(key.prefix)}><Icon name="delete"/></button></Show></div>
        </li>}</For></ul>
      </Show>
    </section>}>
      <ApiKeyEditor accountId={props.accountId} service={service} onCreated={()=>{void refetch();}} onBack={()=>navigation.navigate({section:'access'})}/>
    </Show>
  </div>;
}
function ApiKeyEditor(props:{accountId:string;service:ApiKeyService;onCreated:()=>void;onBack:()=>Promise<unknown>}):JSX.Element {
  const initial=readOnlyScope(props.accountId);
  const [label,setLabel]=createSignal(''),[scope,setScope]=createSignal<ApiKeyScope>(initial);
  const [minted,setMinted]=createSignal<MintedKey|null>(null),[busy,setBusy]=createSignal(false),[error,setError]=createSignal('');
  let disposed=false;onCleanup(()=>{disposed=true;setMinted(null);});
  const guard=useEditorGuard(()=>busy()||!!minted()||!!label()||JSON.stringify(scope())!==JSON.stringify(initial),busy);
  async function create():Promise<void>{
    if(busy()||minted())return;
    if(!label().trim()){setError(t('apikeys-error-need-label'));return;}
    setError('');setBusy(true);
    try{const result=await props.service.create({label:label().trim(),accountId:props.accountId,scope:scope()});if(disposed)return;setMinted(result);setLabel('');setScope(initial);props.onCreated();}
    catch(cause){if(!disposed)toast.error(cause instanceof Error?cause.message:t('apikeys-error-create'));}
    finally{if(!disposed)setBusy(false);}
  }
  async function copy(secret:string):Promise<void>{
    try{if(!navigator.clipboard)return;await navigator.clipboard.writeText(secret);if(!disposed)toast.success(t('apikeys-copied'));}catch{/* The secret remains selectable for manual copying. */}
  }
  return <section class={css.section} aria-label={t('apikeys-create')}>
    <div class={linked.header}><h2 class={css.heading}>{t('apikeys-create')}</h2><button type="button" class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(async()=>{await props.onBack();})}><Icon name="back"/>{t('common-back')}</button></div>
    <p class={css.prose}>{t('apikeys-intro')}</p>
    <Show when={minted()} fallback={<form class={layout.form} onSubmit={event=>{event.preventDefault();void create();}}>
      <fieldset class={layout.fields} disabled={busy()}>
        <label class={css.field}><span class={css.subHeading}>{t('apikeys-label')}</span><input class={`${css.input} ${layout.control}`} value={label()} placeholder={t('apikeys-label-placeholder')} aria-label={t('apikeys-label-aria')} onInput={event=>setLabel(event.currentTarget.value)}/></label>
        <ScopeBuilder scope={scope()} onChange={setScope}/>
        <Show when={error()}><p class={css.error} role="alert">{error()}</p></Show>
        <div class={css.row}><button type="submit" class={css.button}><Icon name="key"/>{t('apikeys-create')}</button><button type="button" class={css.ghost} onClick={()=>void guard.leave(async()=>{await props.onBack();})}>{t('common-cancel')}</button></div>
      </fieldset>
    </form>}>{key=><div class={css.field} data-testid="minted-key"><p class={css.warn}>{t('apikeys-reveal-warning')}</p><code class={css.token} data-testid="minted-token" tabindex="0">{key().displayToken}</code><div class={css.row}><button type="button" class={css.ghost} aria-label={t('apikeys-copy-aria')} onClick={()=>void copy(key().displayToken)}><Icon name="copy"/>{t('apikeys-copy')}</button><button type="button" class={css.button} onClick={()=>{setMinted(null);void props.onBack();}}><Icon name="check"/>{t('apikeys-saved')}</button></div></div>}</Show>
    {guard.dialog()}
  </section>;
}
export default ApiKeys;
