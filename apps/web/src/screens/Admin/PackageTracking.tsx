import {createSignal,onMount,Show,useContext,type JSX} from 'solid-js';
import {ShellContext} from './shell/context.ts';
import {adminRequest} from './shell/adminApi.ts';
import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import {Feedback, describeAdminError} from './shell/Feedback.tsx';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {AdminSubsection} from './AdminSubsection.tsx';
import {loadCatalog,t} from '../../i18n';
import * as css from './admin.css.ts';
interface Provider {enabled:boolean;refreshSecs:number;configured:boolean;revision:string}
export function PackageTracking():JSX.Element {
  const shell=useContext(ShellContext);
  const readOnly=()=>shell?.can.can('integrations.write')!==true;
  const [ready,setReady]=createSignal(false);const [enabled,setEnabled]=createSignal(false);const [interval,setInterval]=createSignal(900);const [key,setKey]=createSignal('');const [configured,setConfigured]=createSignal(false);const [etag,setEtag]=createSignal('');const [baseline,setBaseline]=createSignal('');const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [saved,setSaved]=createSignal(false);
  const snapshot=()=>JSON.stringify([enabled(),interval()]);
  useDirtyGuard(()=>ready()&&(key()!==''||snapshot()!==baseline()));
  function accept(provider:Provider|null,revision:string):void {setEnabled(provider?.enabled??false);setInterval(provider?.refreshSecs??900);setConfigured(provider?.configured??false);setKey('');setEtag(revision);setReady(true);setBaseline(snapshot());}
  async function load():Promise<void>{setBusy(true);setError('');try{const result=await adminRequest<{provider:Provider|null}>('/package-tracking');accept(result.data.provider,result.etag??'');}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}}
  async function save(event:SubmitEvent):Promise<void>{event.preventDefault();if(busy()||readOnly())return;setBusy(true);setError('');setSaved(false);try{const result=await adminRequest<{provider:Provider}>('/package-tracking',{method:'PUT',ifMatch:etag(),body:{enabled:enabled(),refreshSecs:interval(),...(key()?{apiKey:key()}:{})}});accept(result.data.provider,result.etag??'');setSaved(true);}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}}
  onMount(()=>{void loadCatalog('packages');void load();});
  return <section class={css.subsectionBody} aria-label={t('packages-admin-title')}><h2>{t('packages-admin-title')}</h2><p>{t('packages-admin-help')}</p><Show when={error()}><p role="alert">{error()}</p></Show><Feedback kind="success" as="toast" message={saved()?t('packages-admin-saved'):null}/>
    <Show when={ready()} fallback={<Show when={busy()} fallback={<button class="btn" onClick={()=>void load()}>{t('packages-reload')}</button>}><AsyncPending/></Show>}><form class={css.subsectionBody} onSubmit={event=>void save(event)}><AdminSubsection title={t('packages-admin-provider')} description={t('packages-admin-provider-help')}>
      <label><input type="checkbox" checked={enabled()} disabled={busy()||readOnly()} onChange={event=>setEnabled(event.currentTarget.checked)}/>{t('packages-admin-enabled')}</label>
      <label class="field"><span>{t('packages-admin-key')}</span><input type="password" autocomplete="new-password" maxlength={512} required={!configured()} value={key()} disabled={busy()||readOnly()} onInput={event=>setKey(event.currentTarget.value)} aria-describedby="package-key-help"/><small id="package-key-help">{t(configured()?'packages-admin-key-retain':'packages-admin-key-help')}</small></label>
      <label class="field"><span>{t('packages-admin-interval')}</span><input type="number" min={60} max={86400} required value={interval()} disabled={busy()||readOnly()} onInput={event=>setInterval(Number(event.currentTarget.value))} aria-describedby="package-interval-help"/><small id="package-interval-help">{t('packages-admin-interval-help')}</small></label>
    </AdminSubsection><div><button class="btn" type="submit" disabled={busy()||readOnly()||(!key()&&snapshot()===baseline())}>{t('packages-admin-save')}</button></div></form></Show>
  </section>;
}
