import {Show,createSignal,onMount,type JSX} from 'solid-js';
import {nextcloudAccountsApi,type NextcloudAccountsApi,type CloudInstance,type CloudWrite} from '../../api/nextcloudAccounts.ts';
import {loadCatalog,t} from '../../i18n';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {useShell} from './shell/context.ts';
import {sectionPath} from './shell/router.ts';
import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import {describeAdminError} from './shell/Feedback.tsx';
import {Icon} from '../../components/Icon.tsx';
import {DataTable} from './shell/DataTable.tsx';
import * as layout from './integrationList.css.ts';
import {AdminSubsection} from './AdminSubsection.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';

export function AdminNextcloud(props:{api?:NextcloudAccountsApi}):JSX.Element {
  const api=()=>props.api??nextcloudAccountsApi;const shell=useShell();
  const [rows,setRows]=createSignal<CloudInstance[]>();const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');
  const route=()=>shell.router.route();const editing=()=>route().rest[0];const selected=()=>rows()?.find(row=>row.id===editing());
  const writable=()=>shell.can.can('integrations.write');
  const go=(id?:string):void=>{void shell.router.navigate(sectionPath('nextcloud',id?[id]:[]));};
  async function reload():Promise<void>{setBusy(true);setError('');try{setRows((await api().instances()).instances);}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}}
  onMount(()=>{void loadCatalog('nextcloud');void reload();});
  return <section class={css.section}>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <Show when={rows()} fallback={<Show when={busy()} fallback={<button class="btn" onClick={()=>void reload()}>{t('nextcloud-account-reload')}</button>}><AsyncPending/></Show>}>
      <Show when={editing()} fallback={<>
        <h2 class={css.heading}>{t('nextcloud-admin-title')}</h2><p class={css.note}>{t('nextcloud-admin-help')}</p>
        <div class={ux.pageActions}><button class="btn btn--primary" disabled={!writable()||busy()||(rows()?.length??0)>=64} onClick={()=>go('new')}>{t('nextcloud-admin-add')}</button><button class="btn" disabled={busy()} onClick={()=>void reload()}>{t('nextcloud-account-reload')}</button></div>
        <DataTable caption={t('nextcloud-admin-title')} rows={rows()??[]} rowKey={row=>row.id} state={busy()?'loading':'ready'} emptyText={t('nextcloud-admin-empty')}
          columns={[{id:'name',header:t('nextcloud-admin-name'),cell:row=><span class={layout.value} title={row.name}>{row.name}</span>},{id:'url',header:t('nextcloud-admin-url'),cell:row=><span class={layout.value} title={row.baseUrl}>{row.baseUrl}</span>},{id:'status',header:t('nextcloud-admin-status'),cell:row=>t(row.enabled?'nextcloud-admin-enabled':'nextcloud-admin-disabled')}]}
          rowActions={row=><button type="button" class={ux.iconAction} aria-label={t('nextcloud-admin-open',{name:row.name})} data-tooltip={t('nextcloud-admin-edit')} onClick={()=>go(row.id)}><Icon name="edit"/></button>}/>
        <p>{t('nextcloud-admin-private-help')}</p>
      </>}>
        <Show when={editing()==='new'} fallback={<Show when={selected()} keyed fallback={<p role="alert">{t('nextcloud-admin-missing')}</p>}>{row=><InstanceEditor initial={row} writable={writable()} api={api()} onDone={async()=>{await reload();go();}} onCancel={()=>go()}/>}</Show>}>
          <InstanceEditor writable={writable()} api={api()} onDone={async()=>{await reload();go();}} onCancel={()=>go()}/>
        </Show>
      </Show>
    </Show>
  </section>;
}
function InstanceEditor(props:{initial?:CloudInstance;writable:boolean;api:NextcloudAccountsApi;onDone:()=>Promise<void>;onCancel:()=>void}):JSX.Element {
  const initial:CloudWrite={name:props.initial?.name??'',baseUrl:props.initial?.baseUrl??'',enabled:props.initial?.enabled??false,caPem:props.initial?.caPem??''};
  const [draft,setDraft]=createSignal(initial);const [baseline,setBaseline]=createSignal(JSON.stringify(initial));const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');
  useDirtyGuard(()=>JSON.stringify(draft())!==baseline());
  const change=(value:Partial<CloudWrite>):void=>{setDraft(previous=>({...previous,...value}));};
  async function save():Promise<void>{if(busy()||!props.writable)return;setBusy(true);setError('');try{await props.api.save(props.initial?.id??crypto.randomUUID(),draft(),props.initial?.etag);setBaseline(JSON.stringify(draft()));await props.onDone();}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}}
  return <><h2 class={css.heading}>{t(props.initial?'nextcloud-admin-edit':'nextcloud-admin-add')}</h2><p>{t('nextcloud-admin-revalidation')}</p>
    <form class={css.subsectionBody} onSubmit={event=>{event.preventDefault();void save();}}><AdminSubsection title={t('nextcloud-admin-authority')}>
      <label class="field"><span>{t('nextcloud-admin-name')}</span><input required maxlength={128} disabled={busy()||!props.writable} value={draft().name} onInput={event=>change({name:event.currentTarget.value})}/><small>{t('nextcloud-admin-name-help')}</small></label>
      <label class="field"><span>{t('nextcloud-admin-url')}</span><input type="url" required maxlength={2048} disabled={busy()||!props.writable} value={draft().baseUrl} onInput={event=>change({baseUrl:event.currentTarget.value})}/><small>{t('nextcloud-admin-url-help')}</small></label>
      <label class="field"><span>{t('nextcloud-admin-ca')}</span><textarea rows={6} maxlength={65536} disabled={busy()||!props.writable} value={draft().caPem??''} onInput={event=>change({caPem:event.currentTarget.value})}/><small>{t('nextcloud-admin-ca-help')}</small></label>
      <label><input type="checkbox" disabled={busy()||!props.writable} checked={draft().enabled} onChange={event=>change({enabled:event.currentTarget.checked})}/> {t('nextcloud-admin-enable')}</label>
      <div class={ux.pageActions}><button class="btn btn--primary" disabled={busy()||!props.writable||JSON.stringify(draft())===baseline()} type="submit">{t('common-save')}</button><button class="btn" disabled={busy()} type="button" onClick={props.onCancel}>{t('common-cancel')}</button></div>
    </AdminSubsection></form><Show when={error()}><p role="alert">{error()}</p></Show>
  </>;
}
