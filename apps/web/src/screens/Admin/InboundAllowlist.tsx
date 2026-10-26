import {For,Show,createSignal,createUniqueId,onMount,type JSX} from 'solid-js';
import {inboundAllowlistApi,type InboundAllowlistApi,type InboundConfig,type InboundView} from '../../api/admin/inboundAllowlist.ts';
import {loadCatalog,t} from '../../i18n';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {useShell} from './shell/context.ts';
import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {describeAdminError} from './shell/Feedback.tsx';
import {DataTable} from './shell/DataTable.tsx';
import {AdminSubsection} from './AdminSubsection.tsx';
import {dnsLimits,inboundDraft,inboundConfig,inboundProblem,type InboundDraft} from './inboundAllowlistModel.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
import * as list from './integrationList.css.ts';

export function InboundAllowlist(props:{api?:InboundAllowlistApi}):JSX.Element {
 const formId=createUniqueId();
 const shell=useShell();const api=()=>props.api??inboundAllowlistApi;
 const [view,setView]=createSignal<InboundView>();const [draft,setDraft]=createSignal<InboundDraft>();
 const [baseline,setBaseline]=createSignal('');const [etag,setEtag]=createSignal('');
 const [busy,setBusy]=createSignal(false),[error,setError]=createSignal(''),[saved,setSaved]=createSignal(false),[conflict,setConflict]=createSignal(false);
 const [pending,setPending]=createSignal<InboundConfig>();
 const unscoped=()=>{const grants=shell.can.grants();return grants.kind==='all'?grants.reason==='no-rbac':grants.me.domainScope===null;};
 const readable=()=>unscoped()&&shell.can.can('security.read');
 const writable=()=>unscoped()&&shell.can.can('security.write');
 const dirty=()=>!!draft()&&JSON.stringify(draft())!==baseline();useDirtyGuard(dirty);
 function accept(value:InboundView,revision:string|null):void{setView(value);const next=inboundDraft(value.config);setDraft(next);setBaseline(JSON.stringify(next));setEtag(revision??'');setConflict(false);}
 async function reload():Promise<void>{if(!readable()||busy())return;setBusy(true);setError('');try{const result=await api().read();if(dirty()){setView(result.data);setConflict(result.etag!==etag());}else accept(result.data,result.etag);}catch(cause){setError(describeAdminError(cause));}finally{setBusy(false);}}
 const change=(next:InboundDraft):void=>{setDraft(next);setSaved(false);};
 function submit(event:SubmitEvent):void{event.preventDefault();if(!writable()||busy()||!etag()||!dirty())return;const value=inboundConfig(draft()!);const problem=inboundProblem(value);if(problem){setError(t(problem));return;}setError('');setPending(value);}
 async function save():Promise<void>{const value=pending();if(!value||!writable()||!etag())return;setBusy(true);try{const result=await api().save(value,etag());accept(result.data,result.etag);setPending(undefined);setSaved(true);}finally{setBusy(false);}}
 onMount(()=>{void loadCatalog('admin-inbound');void reload();});
 return <section class={css.section}>
  <h2 class={css.heading}>{t('admin-inbound-title')}</h2><p class={css.note}>{t('admin-inbound-help')}</p>
  <Show when={readable()} fallback={<p role="alert">{t('admin-inbound-unscoped')}</p>}>
   <Show when={error()}><p role="alert">{error()}</p></Show><Show when={saved()}><p role="status">{t('admin-inbound-saved')}</p></Show>
   <Show when={conflict()}><p role="alert">{t('admin-inbound-conflict')}</p></Show>
   <div class={ux.pageActions}><button type="button" class="btn" disabled={busy()} onClick={()=>void reload()}>{t('admin-inbound-refresh')}</button><Show when={dirty()}><button type="button" class="btn" disabled={busy()} onClick={()=>{setDraft(JSON.parse(baseline()) as InboundDraft);setSaved(false);}}>{t('admin-inbound-reset')}</button></Show></div>
   <Show when={draft()} fallback={<Show when={busy()}><AsyncPending/></Show>}>{value=><form class={css.subsectionBody} onSubmit={submit}>
    <For each={['admin','webmail'] as const}>{surface=><AdminSubsection title={t(`admin-inbound-${surface}`)} description={t(`admin-inbound-${surface}-help`)}>
     <label class="field"><input type="checkbox" disabled={!writable()||busy()} checked={value()[surface].enabled} onChange={event=>change({...value(),[surface]:{...value()[surface],enabled:event.currentTarget.checked}})}/><span>{t('admin-inbound-enabled')}</span></label>
     <label class="field"><span>{t('admin-inbound-cidrs')}</span><textarea rows={4} maxlength={8448} disabled={!writable()||busy()} value={value()[surface].cidrs} onInput={event=>change({...value(),[surface]:{...value()[surface],cidrs:event.currentTarget.value}})}/><small>{t('admin-inbound-cidrs-help')}</small></label>
     <label class="field"><span>{t('admin-inbound-hostnames')}</span><textarea rows={4} maxlength={16384} disabled={!writable()||busy()} value={value()[surface].hostnames} onInput={event=>change({...value(),[surface]:{...value()[surface],hostnames:event.currentTarget.value}})}/><small>{t('admin-inbound-hostnames-help')}</small></label>
    </AdminSubsection>}</For>
    <AdminSubsection title={t('admin-inbound-dns')} description={t('admin-inbound-dns-help')}>
     <label class="field"><span>{t('admin-inbound-servers')}</span><textarea rows={3} maxlength={512} disabled={!writable()||busy()} value={value().dns.servers} onInput={event=>change({...value(),dns:{...value().dns,servers:event.currentTarget.value}})}/><small>{t('admin-inbound-servers-help')}</small></label>
     <div class={css.grid}><For each={Object.entries(dnsLimits)}>{([key,bounds])=><label class="field"><span>{t(`admin-inbound-dns-${key}`)}</span><input type="number" aria-label={t(`admin-inbound-dns-${key}`)} aria-describedby={`${formId}-${key}-bounds`} required min={bounds[0]} max={bounds[1]} step="1" disabled={!writable()||busy()} value={value().dns[key as keyof typeof dnsLimits]} onInput={event=>change({...value(),dns:{...value().dns,[key]:event.currentTarget.valueAsNumber}})}/><small id={`${formId}-${key}-bounds`}>{t('admin-inbound-bounds',{min:bounds[0],max:bounds[1]})}</small></label>}</For></div>
    </AdminSubsection>
    <p class={css.note}>{t('admin-inbound-lockout')}</p><Show when={!etag()}><p role="alert">{t('admin-inbound-no-revision')}</p></Show>
    <div class={ux.pageActions}><button type="submit" class="btn btn--primary" disabled={busy()||!writable()||!dirty()||!etag()||conflict()}>{t('admin-inbound-review')}</button></div>
   </form>}</Show>
   <Show when={view()}>{value=><AdminSubsection title={t('admin-inbound-status')} description={t('admin-inbound-status-help')}>
    <p role="status">{t(value().status.refreshing?'admin-inbound-refreshing':'admin-inbound-idle')}</p>
    <DataTable caption={t('admin-inbound-status')} rows={value().status.hosts} rowKey={row=>row.hostname} state="ready" emptyText={t('admin-inbound-no-hosts')} columns={[
      {id:'hostname',header:t('admin-inbound-hostname'),cell:row=><span class={list.value} title={row.hostname}>{row.hostname}</span>},
      {id:'state',header:t('admin-inbound-state'),cell:row=>t(row.usable?'admin-inbound-usable':'admin-inbound-unavailable')},
      {id:'addresses',header:t('admin-inbound-addresses'),cell:row=><span class={list.value} title={row.addresses?.join(', ')}>{row.addresses?.join(', ')||'—'}</span>},
      {id:'validity',header:t('admin-inbound-validity'),cell:row=>row.validForSeconds===null?'—':t('admin-inbound-seconds',{seconds:row.validForSeconds})},
      {id:'error',header:t('admin-inbound-error'),cell:row=>row.error||'—'},
    ]}/>
   </AdminSubsection>}</Show>
  </Show>
  <ConfirmDialog open={pending()!==undefined} title={t('admin-inbound-confirm-title')} body={t('admin-inbound-lockout')} typedConfirmation={t('admin-inbound-confirm-word')} confirmLabel={t('admin-inbound-save')} onCancel={()=>setPending(undefined)} onConfirm={save}>
   <Show when={pending()}>{value=><For each={['admin','webmail'] as const}>{surface=><p><strong>{t(`admin-inbound-${surface}`)}</strong>: {t(value()[surface].enabled?'admin-inbound-on':'admin-inbound-off')} · {t('admin-inbound-cidrs')}: {value()[surface].cidrs.join(', ')||'—'}</p>}</For>}</Show>
  </ConfirmDialog>
 </section>;
}
