import { createSignal, onMount, Show, useContext, type JSX } from 'solid-js';
import { automaticLocksApi, type AutomaticLock, type AutomaticLocksApi } from '../../api/admin/automaticLocks.ts';
import { Icon } from '../../components/Icon.tsx';
import { t } from '../../i18n';
import { formatDateTime } from '../../i18n/format.ts';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { DataTable, type Column } from './shell/DataTable.tsx';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import * as ux from './shell/ux.css.ts';
import * as css from './admin.css.ts';
export function AutomaticLocks(props:{api?:AutomaticLocksApi}):JSX.Element {
 const shell=useContext(ShellContext); const api=()=>props.api??automaticLocksApi;
 const [rows,setRows]=createSignal<AutomaticLock[]>([]);const [loading,setLoading]=createSignal(true);
 const [error,setError]=createSignal('');const [query,setQuery]=createSignal('');
 const [page,setPage]=createSignal(0);
 const [surface,setSurface]=createSignal<'all'|'admin'|'webmail'>('all');
 const [selected,setSelected]=createSignal<AutomaticLock>();const [success,setSuccess]=createSignal('');
 async function reload():Promise<void>{setLoading(true);try{setRows(await api().list());setPage(0);setError('');}catch(cause){setError(describeAdminError(cause));}finally{setLoading(false);}}
 onMount(()=>void reload());
 const label=(row:AutomaticLock):string=>t(`admin-obs-lock-surface-${row.surface}`);
 const visible=():AutomaticLock[]=>rows().filter(row=>(surface()==='all'||row.surface===surface())&&`${row.ip} ${label(row)}`.toLowerCase().includes(query().trim().toLowerCase()));
 const columns=():Column<AutomaticLock>[]=>[
 {id:'ip',header:t('admin-obs-ban-ip'),cell:row=><span class={css.mono}>{row.ip}</span>},
 {id:'surface',header:t('admin-obs-lock-surface'),cell:label},
 {id:'count',header:t('admin-obs-lock-failures'),cell:row=>row.failures},
 {id:'until',header:t('admin-obs-lock-until'),cell:row=>formatDateTime(new Date(row.lockedUntil*1000).toISOString())},
 ];
 return <section class={css.section} aria-label={t('admin-obs-locks-title')}>
 <div class={ux.pageHead}><h2 class={css.heading}>{t('admin-obs-locks-title')}</h2><div class={ux.pageActions}>
 <button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('observability'))}>{t('admin-obs-locks-back')}</button>
 <button class="btn" disabled={loading()} onClick={()=>void reload()}>{t('admin-obs-locks-refresh')}</button></div></div>
 <p class={css.note}>{t('admin-obs-locks-note')}</p>
 <Show when={shell?.can.can('settings.read')}><button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(`${sectionPath('settings')}?field=security.signin.adminFailureLimit`)}>{t('admin-obs-locks-configure')}</button></Show>
 <Feedback kind="success" as="toast" message={success()}/>
 <div class={css.card}><label class="field">{t('admin-obs-lock-surface')} <select value={surface()} onChange={event=>{const value=event.currentTarget.value;if(value==='all'||value==='admin'||value==='webmail'){setSurface(value);setPage(0);}}}>
 <option value="all">{t('admin-obs-lock-surface-all')}</option><option value="admin">{t('admin-obs-lock-surface-admin')}</option><option value="webmail">{t('admin-obs-lock-surface-webmail')}</option></select></label>
 <DataTable caption={t('admin-obs-locks-title')} columns={columns()} rows={visible().slice(page()*50,(page()+1)*50)} paging={{hasPrev:page()>0,hasNext:(page()+1)*50<visible().length,onPrev:()=>setPage(value=>Math.max(0,value-1)),onNext:()=>setPage(value=>value+1)}} rowKey={row=>`${row.surface}:${row.ip}`} state={loading()?'loading':error()?'error':'ready'} errorText={error()} onRetry={()=>void reload()} emptyText={t('admin-obs-locks-empty')} search={{label:t('admin-obs-locks-search'),value:query(),onSearch:value=>{setQuery(value);setPage(0);},debounceMs:0}}
 rowActions={row=><Show when={shell?.can.can('bans.write')}><button class="btn btn--ghost btn--sm" aria-label={t('admin-obs-lock-unlock-target',{ip:row.ip,surface:label(row)})} onClick={()=>setSelected(row)}><Icon name="unlock"/>{t('admin-obs-lock-unlock')}</button></Show>}/></div>
 <ConfirmDialog open={selected()!==undefined} title={t('admin-obs-lock-unlock-target',{ip:selected()?.ip??'',surface:selected()?label(selected()!):''})} body={t('admin-obs-lock-confirm')} confirmLabel={t('admin-obs-lock-unlock')} onCancel={()=>setSelected(undefined)} onConfirm={async()=>{const row=selected();if(!row)return;await api().unlock(row);setSelected(undefined);setSuccess(t('admin-obs-lock-unlocked',{ip:row.ip,surface:label(row)}));await reload();}}/>
 </section>;
}
