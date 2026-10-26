import {For,Show,createSignal,onMount,type JSX} from 'solid-js';
import {egressDestinationsApi,type EgressDestinationsApi,type InternalDestination,type DestinationDraft} from '../../api/admin/egressDestinations.ts';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {t} from '../../i18n';
import {describeAdminError} from './shell/Feedback.tsx';
import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
import { field, editor } from './Egress.css.ts';

export function EgressDestinations(props:{api?:EgressDestinationsApi;editor?:string;readOnly:boolean;onNavigate:(editor?:string)=>void;onBack:()=>void}):JSX.Element {
  const api=()=>props.api??egressDestinationsApi;
  const [rows,setRows]=createSignal<InternalDestination[]>();const [classes,setClasses]=createSignal<string[]>([]);
  const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [removing,setRemoving]=createSignal<string>();
  async function load():Promise<void>{setError('');try{const result=await api().list();setRows(result.destinations);setClasses(result.eligibleClasses);}catch(error){setError(describeAdminError(error));}}
  onMount(()=>void load());
  async function remove(id:string):Promise<void>{if(busy()||props.readOnly)return;setBusy(true);setError('');try{await api().remove(id);setRemoving(undefined);await load();}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}}
  return <section class={css.section}><h2>{t('admin-egress-destinations-title')}</h2><p>{t('admin-egress-destinations-help')}</p>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <Show when={rows()} fallback={<Show when={!error()} fallback={<button class="btn" onClick={()=>void load()}>{t('common-retry')}</button>}><AsyncPending/></Show>}>
      <Show when={props.editor==='new'} fallback={<>
        <div class={ux.pageActions}><button class="btn" onClick={props.onBack}>{t('admin-egress-destinations-back')}</button><button class="btn btn--primary" disabled={props.readOnly||busy()} onClick={()=>props.onNavigate('new')}>{t('admin-egress-destinations-add')}</button></div>
        <div class={css.tableWrap}><table class={css.table}><thead><tr><th>{t('admin-egress-destination-class')}</th><th>{t('admin-egress-destination-target')}</th><th>{t('admin-egress-destination-note')}</th><th>{t('admin-egress-destination-actions')}</th></tr></thead><tbody><For each={rows()}>{row=><tr><td>{t(`admin-egress-class-${row.trafficClass}`)}</td><td>{row.destination}</td><td>{row.note}</td><td><Show when={removing()===row.id} fallback={<button class="btn" disabled={props.readOnly||busy()} onClick={()=>setRemoving(row.id)}>{t('admin-egress-destination-revoke')}</button>}><p>{t('admin-egress-destination-confirm')}</p><button class="btn" disabled={busy()} onClick={()=>void remove(row.id)}>{t('admin-egress-destination-confirm-button')}</button><button class="btn" disabled={busy()} onClick={()=>setRemoving(undefined)}>{t('common-cancel')}</button></Show></td></tr>}</For></tbody></table></div>
        <Show when={rows()?.length===0}><p>{t('admin-egress-destinations-empty')}</p></Show>
      </>}><DestinationEditor classes={classes()} readOnly={props.readOnly} api={api()} onCancel={()=>props.onNavigate()} onSaved={async()=>{await load();props.onNavigate();}}/></Show>
    </Show>
  </section>;
}

function DestinationEditor(props:{classes:string[];readOnly:boolean;api:EgressDestinationsApi;onCancel:()=>void;onSaved:()=>Promise<void>}):JSX.Element {
  const [draft,setDraft]=createSignal<DestinationDraft>({trafficClass:'',destination:'',note:''});
  const [saved,setSaved]=createSignal(false);const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');
  useDirtyGuard(()=>!saved()&&Object.values(draft()).some(Boolean));
  const change=(patch:Partial<DestinationDraft>):void=>{setDraft(value=>({...value,...patch}));};
  async function save():Promise<void>{if(busy()||props.readOnly)return;setBusy(true);setError('');try{await props.api.create(draft());setSaved(true);await props.onSaved();}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}}
  return <form class={`${css.subsectionBody} ${editor}`} onSubmit={event=>{event.preventDefault();void save();}}>
    <label class={field}><span>{t('admin-egress-destination-class')}</span><select required disabled={busy()||props.readOnly} value={draft().trafficClass} onChange={event=>change({trafficClass:event.currentTarget.value})}><option value="">{t('admin-egress-destination-choose')}</option><For each={props.classes}>{value=><option value={value}>{t(`admin-egress-class-${value}`)}</option>}</For></select><small>{t('admin-egress-destination-class-help')}</small></label>
    <label class={field}><span>{t('admin-egress-destination-target')}</span><input required maxlength={512} disabled={busy()||props.readOnly} value={draft().destination} onInput={event=>change({destination:event.currentTarget.value})}/><small>{t('admin-egress-destination-target-help')}</small></label>
    <label class={field}><span>{t('admin-egress-destination-note')}</span><input maxlength={256} disabled={busy()||props.readOnly} value={draft().note} onInput={event=>change({note:event.currentTarget.value})}/><small>{t('admin-egress-destination-note-help')}</small></label>
    <div class={ux.pageActions}><button class="btn btn--primary" type="submit" disabled={busy()||props.readOnly||!draft().trafficClass||!draft().destination.trim()}>{t('common-save')}</button><button class="btn" type="button" disabled={busy()} onClick={props.onCancel}>{t('common-cancel')}</button></div>
    <Show when={error()}><p role="alert">{error()}</p></Show>
  </form>;
}
