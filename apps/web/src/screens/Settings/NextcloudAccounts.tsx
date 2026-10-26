import {For,Show,createSignal,onMount,onCleanup,type JSX} from 'solid-js';
import {nextcloudAccountsApi,type NextcloudAccountsApi,type CloudAccount,type CloudInstance} from '../../api/nextcloudAccounts.ts';
import {observePreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {loadCatalog,t} from '../../i18n';
import {useEditorGuard,useEntityNavigation} from './entityEditor.tsx';
import * as css from './styles.css.ts';
import {NextcloudPim} from './NextcloudPim.tsx';
import * as layout from './editor.css.ts';

export function NextcloudAccounts(props:{api?:NextcloudAccountsApi}):JSX.Element {
  const api=()=>props.api??nextcloudAccountsApi;const nav=useEntityNavigation('nextcloud');
  let generation=0;let activeAccount:string|undefined;
  const [view,setView]=createSignal<CloudAccount>();const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');
  async function reload():Promise<void>{const epoch=++generation;setBusy(true);setError('');try{const next=await api().account();if(epoch===generation&&(!activeAccount||activeAccount===next.accountId))setView(next);}catch{if(epoch===generation)setError(t('nextcloud-account-load-error'));}finally{if(epoch===generation)setBusy(false);}}
  async function remove(id:string,etag:string):Promise<void>{setBusy(true);setError('');try{await api().unlink(id,view()!.accountId,etag);await reload();}catch{setError(t('nextcloud-account-unlink-error'));}finally{setBusy(false);}}
  onMount(()=>{void loadCatalog('nextcloud');if(props.api){void reload();return;}const stop=observePreferenceAccount(account=>{generation++;setView(undefined);setBusy(false);activeAccount=account?.id;if(account)void reload();});onCleanup(()=>{generation++;stop();});});
  const editing=()=>nav.route().editor?.id;
  const selected=()=>view()?.instances.find(instance=>instance.id===(editing()?.startsWith('pim:')?editing()!.slice(4):editing()));
  return <section class={css.section}>
    <Show when={editing()} fallback={<>
      <h2 class={css.heading}>{t('nextcloud-account-title')}</h2><p>{t('nextcloud-account-help')}</p>
      <Show when={busy()&&!view()}><AsyncPending/></Show>
      <Show when={error()}><p role="alert">{error()}</p><button class={css.button} onClick={()=>void reload()}>{t('nextcloud-account-reload')}</button></Show>
      <Show when={view()}>{data=><><Show when={data().instances.length===0}><p>{t('nextcloud-account-no-instances')}</p></Show>
        <For each={data().instances}>{instance=>{const linked=()=>data().links.find(link=>link.instanceId===instance.id);return <section class={css.section}><h3>{instance.name}</h3><p>{instance.baseUrl}</p><p>{t(linked()?.ready?'nextcloud-account-ready':linked()?'nextcloud-account-stale':'nextcloud-account-unlinked')}</p>
          <div class={css.actions}><button class={css.button} disabled={busy()||!instance.enabled||data().allowed===false} onClick={()=>void nav.navigate({section:'nextcloud',editor:{kind:'edit',id:instance.id}})}>{t(linked()?'nextcloud-account-relink':'nextcloud-account-link')}</button><Show when={linked()}>{link=><button class={css.danger} disabled={busy()} onClick={()=>void remove(instance.id,link().etag)}>{t('nextcloud-account-unlink')}</button>}</Show><Show when={linked()?.ready}><button class={css.button} disabled={busy()} onClick={()=>void nav.navigate({section:'nextcloud',editor:{kind:'edit',id:`pim:${instance.id}`}})}>{t('nextcloud-pim-title')}</button></Show></div>
        </section>;}}</For></>}</Show>
    </>}>
      <Show when={selected()} keyed fallback={<p role="alert">{t('nextcloud-account-no-instances')}</p>}>{instance=><Show when={editing()?.startsWith('pim:')} fallback={<LinkEditor accountId={view()!.accountId} instance={instance} username={view()?.links.find(link=>link.instanceId===instance.id)?.username??''} etag={view()?.links.find(link=>link.instanceId===instance.id)?.etag} api={api()} onDone={async()=>{await nav.navigate({section:'nextcloud'});await reload();}} onCancel={()=>nav.navigate({section:'nextcloud'})}/> }><NextcloudPim accountId={view()!.accountId} instanceId={instance.id} instanceName={instance.name} onBack={()=>void nav.navigate({section:'nextcloud'})}/></Show>}</Show>
    </Show>
  </section>;
}
function LinkEditor(props:{accountId:string;instance:CloudInstance;username:string;etag:string|undefined;api:NextcloudAccountsApi;onDone:()=>Promise<void>;onCancel:()=>Promise<boolean>}):JSX.Element {
  const [username,setUsername]=createSignal(props.username);const [password,setPassword]=createSignal('');const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [saved,setSaved]=createSignal(false);
  let disposed=false;onCleanup(()=>{disposed=true;});
  const guard=useEditorGuard(()=>!saved()&&(username()!==props.username||password()!==''),busy);
  async function save():Promise<void>{if(busy())return;setBusy(true);setError('');try{await props.api.link(props.instance.id,username(),password(),props.accountId,props.etag);if(disposed)return;setPassword('');setSaved(true);await props.onDone();}catch{if(!disposed){setPassword('');setError(t('nextcloud-account-verify-error'));}}finally{if(!disposed)setBusy(false);}}
  return <><h2 class={css.heading}>{t('nextcloud-account-link')} — {props.instance.name}</h2><p>{t('nextcloud-account-password-help')}</p><form onSubmit={event=>{event.preventDefault();void save();}}><fieldset disabled={busy()} class={layout.fields}>
    <label class={css.label}>{t('nextcloud-account-username')}<input class={css.input} required maxlength={256} autocomplete="off" value={username()} onInput={event=>setUsername(event.currentTarget.value)}/></label>
    <label class={css.label}>{t('nextcloud-account-password')}<input class={css.input} required type="password" maxlength={4096} autocomplete="new-password" value={password()} onInput={event=>setPassword(event.currentTarget.value)}/></label>
    <div class={css.actions}><button class={css.button} type="submit">{t('nextcloud-account-verify')}</button><button class={css.ghost} type="button" onClick={()=>void guard.leave(async()=>{await props.onCancel();})}>{t('settings-cancel')}</button></div>
  </fieldset></form><Show when={error()}><p role="alert">{error()}</p></Show>{guard.dialog()}</>;
}
