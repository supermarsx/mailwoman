import {For,Show,createSignal,onMount,useContext,type JSX} from 'solid-js';
import {brandingApi,type Branding,type BrandingApi,type BrandKind} from '../../../api/admin/branding.ts';
import {brandAssetUrl,refreshBranding} from '../../../branding.ts';
import {loadCatalog,t} from '../../../i18n';
import {ShellContext} from '../shell/context.ts';
import {useDirtyGuard} from '../shell/useDirtyGuard.ts';
import {AsyncPending} from '../../../components/AsyncState.tsx';
import {AdminSubsection} from '../AdminSubsection.tsx';
import {describeAdminError} from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import {assetPreview} from '../branding.css.ts';
const slots=[{kind:'logo',key:'logoId',url:'logoUrl',limit:512},{kind:'icon_master',key:'iconMasterId',url:'faviconUrl',limit:1024},{kind:'background',key:'loginBackgroundId',url:'loginBackgroundUrl',limit:2560},{kind:'pwa_192',key:'pwa192Id',url:'pwa192Url',limit:192},{kind:'pwa_512',key:'pwa512Id',url:'pwa512Url',limit:512},{kind:'pwa_maskable',key:'pwaMaskableId',url:'pwaMaskableUrl',limit:512}] as const;
export function BrandingSettings(props:{api?:BrandingApi}):JSX.Element{
 const api=()=>props.api??brandingApi;const shell=useContext(ShellContext);
 const writable=()=>shell?.can.can('branding.manage')!==false;
 const [value,setValue]=createSignal<Branding>();const [etag,setEtag]=createSignal('');const [baseline,setBaseline]=createSignal('');const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [done,setDone]=createSignal(false);
 const dirty=()=>!!value()&&JSON.stringify(value())!==baseline();useDirtyGuard(dirty);
 async function reload(){setBusy(true);setError('');try{const r=await api().read();setValue(r.data);setBaseline(JSON.stringify(r.data));setEtag(r.etag);}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
 onMount(()=>{void loadCatalog('admin-branding');void reload();});
 async function upload(kind:BrandKind,key:keyof Branding['images'],file:File|undefined){if(!file)return;setBusy(true);setError('');setDone(false);try{const id=await api().upload(kind,file);setValue(v=>v&&({...v,images:{...v.images,[key]:id}}));}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
 async function save(e:SubmitEvent){e.preventDefault();const v=value();if(!v)return;setBusy(true);setError('');try{const r=await api().save(v,etag());setValue(r.data);setBaseline(JSON.stringify(r.data));setEtag(r.etag);setDone(true);await refreshBranding();}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
 return <div class={css.subsectionBody}><h2 class={css.heading}>{t('admin-branding-title')}</h2><p>{t('admin-branding-description')}</p><Show when={error()}><p role="alert">{error()}</p></Show><Show when={done()}><p role="status">{t('admin-branding-saved')}</p></Show><Show when={value()} fallback={<Show when={busy()} fallback={<button type="button" class="btn" onClick={()=>void reload()}>{t('admin-branding-reload')}</button>}><AsyncPending/></Show>}>{v=><form onSubmit={e=>void save(e)} class={css.subsectionBody}>
 <AdminSubsection title={t('admin-branding-text')}><label class="field"><span>{t('admin-branding-name')}</span><input required maxlength={64} disabled={!writable()||busy()} value={v().brandName} onInput={e=>{setDone(false);setValue({...v(),brandName:e.currentTarget.value});}}/></label><label class="field"><span>{t('admin-branding-notice')}</span><textarea maxlength={500} disabled={!writable()||busy()} value={v().loginNotice??''} onInput={e=>{setDone(false);setValue({...v(),loginNotice:e.currentTarget.value||null});}}/></label></AdminSubsection>
 <For each={slots}>{slot=><AdminSubsection title={t(`admin-branding-${slot.kind}`)} description={t(slot.kind.startsWith('pwa_')?'admin-branding-pwa-image-help':'admin-branding-image-help',{pixels:slot.limit})}><Show when={v().images[slot.key]===baselineImage(slot.key)&&brandAssetUrl(v().assets[slot.url])}>{url=><img src={url()} alt={slot.kind==='background'?'':v().brandName} class={assetPreview}/>}</Show><Show when={v().images[slot.key]!==baselineImage(slot.key)&&v().images[slot.key]}><p>{t('admin-branding-image-pending')}</p></Show><label class="field"><span>{t('admin-branding-upload')}</span><input type="file" accept="image/png,image/jpeg" disabled={!writable()||busy()} onChange={e=>void upload(slot.kind,slot.key,e.currentTarget.files?.[0])}/></label><button type="button" class="btn btn--ghost" disabled={!writable()||busy()||!v().images[slot.key]} onClick={()=>{setDone(false);setValue({...v(),images:{...v().images,[slot.key]:null}});}}>{t('admin-branding-remove')}</button></AdminSubsection>}</For>
 <div><button class="btn btn--primary" type="submit" disabled={!writable()||busy()||!dirty()}>{t('admin-branding-save')}</button> <button type="button" class="btn btn--ghost" disabled={busy()} onClick={()=>void reload()}>{t('admin-branding-reload')}</button></div></form>}</Show></div>;
 function baselineImage(key:keyof Branding['images']):string|null{try{return (JSON.parse(baseline()) as Branding).images[key]??null;}catch{return null;}}
}
