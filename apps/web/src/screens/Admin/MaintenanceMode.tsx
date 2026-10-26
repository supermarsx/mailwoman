import {createSignal,onMount,Show,type JSX} from 'solid-js';
import {maintenanceModeApi,type MaintenanceModeApi,type MaintenanceConfig} from '../../api/admin/maintenanceMode.ts';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {loadCatalog,t} from '../../i18n';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
export function MaintenanceMode(props:{readOnly:boolean;api?:MaintenanceModeApi}):JSX.Element {
  const api=props.api??maintenanceModeApi;
  const [value,setValue]=createSignal<MaintenanceConfig>();
  const [baseline,setBaseline]=createSignal('');const [etag,setEtag]=createSignal<string|null>(null);
  const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');
  const [saved,setSaved]=createSignal('');const [confirm,setConfirm]=createSignal(false);
  const [invalid,setInvalid]=createSignal(false);
  useDirtyGuard(()=>!!value()&&JSON.stringify(value())!==baseline());
  async function load():Promise<void>{setBusy(true);setError('');try{const result=await api.read();const current={enabled:result.data.enabled,message:result.data.message,retryAfterSecs:result.data.retryAfterSecs};setValue(current);setBaseline(JSON.stringify(current));setEtag(result.etag);setInvalid(result.data.configurationInvalid===true);}catch(cause){setError(describeAdminError(cause));}finally{setBusy(false);}}
  onMount(()=>{void loadCatalog('admin-maintenance');void load();});
  const change=(patch:Partial<MaintenanceConfig>):void=>{setValue(current=>current?{...current,...patch}:current);setSaved('');};
  async function save():Promise<void>{const current=value(),tag=etag();if(!current||!tag||props.readOnly)return;const result=await api.save(current,tag);setValue(result.data);setBaseline(JSON.stringify(result.data));setEtag(result.etag);setInvalid(false);setConfirm(false);setSaved(t('maintenance-mode-saved'));}
  return <section class={css.section}>
    <h2 class={css.heading}>{t('maintenance-mode-title')}</h2><p class={css.note}>{t('maintenance-mode-help')}</p>
    <Feedback kind="error" message={error()}/><Feedback kind="success" as="toast" message={saved()}/>
    <Show when={invalid()}><p role="alert" class={css.error}>{t('maintenance-mode-invalid')}</p></Show>
    <Show when={busy()&&!value()}><AsyncPending/></Show>
    <Show when={error()}><button class="btn" onClick={()=>void load()}>{t('common-retry')}</button></Show>
    <Show when={value()}>{current=><form class={css.subsectionBody} onSubmit={event=>{event.preventDefault();setConfirm(true);}}>
      <p role="status">{t(baseline()&&JSON.parse(baseline()).enabled?'maintenance-mode-active':'maintenance-mode-inactive')}</p>
      <label class="field"><input type="checkbox" checked={current().enabled} disabled={props.readOnly||busy()} onChange={event=>change({enabled:event.currentTarget.checked})}/>{t('maintenance-mode-enabled')}</label>
      <label class="field"><span>{t('maintenance-mode-message')}</span><textarea aria-label={t('maintenance-mode-message')} rows={3} maxlength={1000} value={current().message} disabled={props.readOnly||busy()} onInput={event=>change({message:event.currentTarget.value})}/><small>{t('maintenance-mode-message-help')}</small></label>
      <label class="field"><span>{t('maintenance-mode-retry')}</span><input aria-label={t('maintenance-mode-retry')} type="number" min={1} max={86400} step={1} required value={current().retryAfterSecs} disabled={props.readOnly||busy()} onInput={event=>change({retryAfterSecs:event.currentTarget.valueAsNumber})}/><small>{t('maintenance-mode-retry-help')}</small></label>
      <div class={ux.pageActions}><button type="submit" class="btn btn--primary" disabled={props.readOnly||busy()||!etag()||(!invalid()&&JSON.stringify(current())===baseline())}>{t('common-save')}</button></div>
    </form>}</Show>
    <ConfirmDialog open={confirm()} title={t('maintenance-mode-confirm')} body={t('maintenance-mode-impact')} confirmLabel={t('common-save')} onCancel={()=>setConfirm(false)} onConfirm={save}/>
  </section>;
}
