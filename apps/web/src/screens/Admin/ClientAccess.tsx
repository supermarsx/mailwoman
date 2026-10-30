import {createSignal,createUniqueId,onMount,onCleanup,Show,type JSX} from 'solid-js';
import {clientAccessApi,validClientPsk,validClientSlug,type ClientAccessApi,type ClientAccessChange,type ClientAccessView} from '../../api/admin/clientAccess.ts';
import {loadCatalog,t} from '../../i18n';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {useSavingToast} from './shell/useSavingToast.ts';
import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import * as css from './admin.css.ts';

export function ClientAccess(props:{readOnly:boolean;api?:ClientAccessApi}):JSX.Element{
 const api=props.api??clientAccessApi;
 const id=createUniqueId();
 let generation=0;onCleanup(()=>{generation++;});
 const [view,setView]=createSignal<ClientAccessView>();const [etag,setEtag]=createSignal<string|null>(null);
 const [pskEnabled,setPskEnabled]=createSignal(false),[slugEnabled,setSlugEnabled]=createSignal(false);
 const [slug,setSlug]=createSignal(''),[psk,setPsk]=createSignal(''),[clearPsk,setClearPsk]=createSignal(false),[hours,setHours]=createSignal(24);
 const [loading,setLoading]=createSignal(true),[saving,setSaving]=createSignal(false),[loadError,setLoadError]=createSignal(''),[error,setError]=createSignal(''),[saved,setSaved]=createSignal('');
 useSavingToast(saving,()=>t('admin-client-access-saving'));
 const adopt=(data:ClientAccessView,tag:string|null):void=>{setView(data);setEtag(tag);setPskEnabled(data.pskEnabled);setSlugEnabled(data.slugEnabled);setSlug(data.slug??'');setHours(data.grantHours);setPsk('');setClearPsk(false);};
 async function load():Promise<void>{const version=++generation;setLoading(true);setLoadError('');setError('');setSaved('');try{const result=await api.read();if(version===generation)adopt(result.data,result.etag);}catch(cause){if(version===generation)setLoadError(describeAdminError(cause));}finally{if(version===generation)setLoading(false);}}
 onMount(()=>{void loadCatalog('admin-security');void load();});
 const disabled=():boolean=>props.readOnly||saving()||loading();
 const valid=():boolean=>(!slug()||validClientSlug(slug()))&&(!slugEnabled()||slug()!=='')&&(!psk()||validClientPsk(psk()))&&(!pskEnabled()||(!clearPsk()&&(view()?.pskConfigured===true||psk()!=='')))&&(!clearPsk()||!pskEnabled())&&Number.isInteger(hours())&&hours()>=1&&hours()<=168;
 const dirty=():boolean=>!!view()&&(pskEnabled()!==view()!.pskEnabled||slugEnabled()!==view()!.slugEnabled||slug()!==(view()!.slug??'')||hours()!==view()!.grantHours||psk()!==''||clearPsk());
 useDirtyGuard(dirty);
 async function save(event:Event):Promise<void>{event.preventDefault();const tag=etag();if(disabled()||!tag||!valid()||!dirty())return;const version=++generation;setSaving(true);setError('');setSaved('');
  const change:ClientAccessChange={pskEnabled:pskEnabled(),slugEnabled:slugEnabled(),slug:slug()||null,grantHours:hours(),...(psk()?{psk:psk()}:{}),...(clearPsk()?{clearPsk:true}:{})};
  try{const result=await api.save(change,tag);if(version===generation){adopt(result.data,result.etag);setSaved(t('admin-client-access-saved'));}}catch(cause){if(version===generation)setError(describeAdminError(cause));}finally{if(version===generation)setSaving(false);}
 }
 return <section class={css.section} aria-label={t('admin-client-access-title')}>
  <h2 class={css.heading}>{t('admin-client-access-title')}</h2><p class={css.note}>{t('admin-client-access-help')}</p><p class={css.note}>{t('admin-client-access-scope')}</p>
  <Feedback kind="error" message={loadError()}/><Feedback kind="error" as="toast" message={error()}/><Feedback kind="success" as="toast" message={saved()}/>
  <Show when={loading()}><AsyncPending/></Show><Show when={loadError()}><button class="btn" onClick={()=>void load()}>{t('common-retry')}</button></Show>
  <Show when={view()}>{current=><form class={css.subsectionBody} onSubmit={event=>void save(event)}>
   <p><strong>{t('admin-client-access-entry')}</strong> <code data-testid="client-entry-path">{current().clientEntryPath}</code></p>
   <label><input type="checkbox" checked={slugEnabled()} disabled={disabled()} onChange={event=>setSlugEnabled(event.currentTarget.checked)}/>{t('admin-client-access-slug-enable')}</label>
   <label class="field">{t('admin-client-access-slug')}<input aria-label={t('admin-client-access-slug')} aria-describedby={`${id}-slug-help`} value={slug()} autocomplete="off" spellcheck={false} disabled={disabled()} aria-invalid={slug()!==''&&!validClientSlug(slug())} onInput={event=>setSlug(event.currentTarget.value)}/><small id={`${id}-slug-help`}>{t('admin-client-access-slug-help')}</small></label>
   <label><input type="checkbox" checked={pskEnabled()} disabled={disabled()} onChange={event=>{setPskEnabled(event.currentTarget.checked);if(event.currentTarget.checked)setClearPsk(false);}}/>{t('admin-client-access-psk-enable')}</label>
   <p data-testid="client-psk-status">{t(current().pskConfigured?'admin-client-access-psk-stored':'admin-client-access-psk-empty')}</p>
   <label class="field">{t('admin-client-access-psk')}<input aria-label={t('admin-client-access-psk')} aria-describedby={`${id}-psk-help`} type="password" autocomplete="new-password" value={psk()} disabled={disabled()||clearPsk()} aria-invalid={psk()!==''&&!validClientPsk(psk())} onInput={event=>setPsk(event.currentTarget.value)}/><small id={`${id}-psk-help`}>{t('admin-client-access-psk-help')}</small></label>
   <Show when={current().pskConfigured&&!pskEnabled()}><label><input type="checkbox" checked={clearPsk()} disabled={disabled()} onChange={event=>{setClearPsk(event.currentTarget.checked);if(event.currentTarget.checked)setPsk('');}}/>{t('admin-client-access-psk-clear')}</label></Show>
   <label class="field">{t('admin-client-access-hours')}<input type="number" min="1" max="168" step="1" value={hours()} disabled={disabled()} onInput={event=>setHours(event.currentTarget.valueAsNumber)}/></label>
   <p class={css.note}>{t('admin-client-access-revoke')}</p>
   <Show when={props.readOnly}><p class={css.note}>{t('admin-client-access-readonly')}</p></Show>
   <button type="submit" class="btn btn--primary" disabled={disabled()||!etag()||!valid()||!dirty()} aria-busy={saving()}>{t('admin-client-access-save')}</button>
   <button type="button" class="btn btn--ghost" disabled={saving()||loading()} onClick={()=>void load()}>{t('admin-client-access-reload')}</button>
  </form>}</Show>
 </section>;
}
