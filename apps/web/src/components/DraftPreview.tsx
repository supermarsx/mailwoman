import {attachmentDecision,attachmentPolicyUrl,loadAttachmentPolicy} from '../api/attachmentPolicy.ts';
import {createResource,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {settings} from '../api/settings.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {apiFetch} from '../api/http.ts';
import {formatBytes,formatDateTime,loadCatalog,t} from '../i18n';
import {sanitizeBody} from '../sanitize/index.ts';
import {bodyFrameDoc,escapeHtml} from '../viewers/sandbox.ts';
import type {ReaderAppearance} from '../viewers/readerAppearance.ts';
import {isReaderColorChoice,resolveReaderColorMode} from '../viewers/readerColorMode.ts';
import {themeCssVars} from '../theme/themeCssVars.ts';
import {buildDownloadUrl} from '../viewers/attachments.ts';
import {createMaxSecurityStore,allowsOriginalBytes} from '../viewers/max-security.ts';
import {mailFeature} from '../state/mailFeatures.ts';
import type {StoredDraft} from './compose/drafts-store.ts';
import {ReaderZoomBox} from './ReaderZoomBox.tsx';
import {Icon} from './Icon.tsx';
import {MessageLoading} from './MessageLoading.tsx';
import * as css from './draftPreview.css.ts';

/** The usual reader frame and typography, without sent-message actions or receipts. */
export function DraftPreview(props:{draft:StoredDraft;onEdit:()=>void;onClose:()=>void;onRefresh:()=>void;active?:boolean}):JSX.Element{
 const app=useApp(),owner=preferenceAccount(),security=createMaxSecurityStore();
 const current=()=>!!owner&&preferenceAccountIsCurrent(owner)&&owner.id===props.draft.accountId&&app.accountId()===owner.id;
 const mode=()=>security.effectiveMode(props.draft.fromOverride??app.identities().find(row=>row.id===props.draft.identityId)?.email??'');
 const [filePolicy]=createResource(()=>current()&&!!((props.draft.attachments?.length??0)+(props.draft.inline?.length??0)),async()=>{try{return await loadAttachmentPolicy();}catch{return null;}});
 const decision=(file:NonNullable<StoredDraft['attachments']>[number])=>{const policy=filePolicy();return policy?attachmentDecision(policy,{name:file.name,mime:file.contentType??'application/octet-stream',size:file.size}):null;};
 let controller:AbortController|undefined;
 onCleanup(()=>controller?.abort());
 onMount(()=>{void loadCatalog('mail');void loadCatalog('reader');});
 const [html]=createResource(()=>filePolicy.loading?false:[props.draft,mode(),filePolicy()] as const,async([draft,securityMode,policy])=>{
  controller?.abort();const abort=new AbortController();controller=abort;
  if(!current())return '';
  let safe:string;
  try{safe=await sanitizeBody({op:'readerSafe',html:draft.bodyFormat==='plain'?`<pre>${escapeHtml(draft.bodyText)}</pre>`:draft.bodyHtml,blockTrackers:true,cleanLinks:true});}
  catch{return `<pre>${escapeHtml(draft.bodyText)}</pre>`;}
  if(!current()||abort.signal.aborted)return '';
  const doc=new DOMParser().parseFromString(safe,'text/html');
  // Draft content cannot authorize remote images, link navigation or tracking.
  for(const link of doc.querySelectorAll('a'))link.removeAttribute('href');
  const images=new Map<string,string>();
  const referenced=new Set([...doc.querySelectorAll('img[src]')].map(img=>img.getAttribute('src')??'').filter(src=>/^cid:/i.test(src)).map(src=>{try{return decodeURIComponent(src.slice(4)).replace(/^<|>$/g,'');}catch{return '';}}));
  const inline=(draft.inline??[]).filter(image=>referenced.has(image.cid.replace(/^<|>$/g,''))&&/^image\/(png|jpe?g|gif|webp|avif|bmp)$/i.test(image.type)&&!!policy&&attachmentDecision(policy,{name:image.name,mime:image.type,size:image.size}).preview);
  const template=app.downloadUrl();let index=0;
  if(template&&securityMode==='full-sanitized')await Promise.all(Array.from({length:Math.min(4,inline.length)},async()=>{
   while(index<inline.length&&!abort.signal.aborted&&current()){
    const image=inline[index++]!;
    try{const response=await apiFetch(buildDownloadUrl(template,{accountId:draft.accountId,blobId:image.blobId,name:image.name,mime:image.type}),{credentials:'same-origin',signal:abort.signal});if(!response.ok)continue;const blob=await response.blob();if(!current()||abort.signal.aborted)continue;
     const data=await new Promise<string>((resolve,reject)=>{const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));reader.onerror=()=>reject(reader.error);reader.readAsDataURL(new Blob([blob],{type:image.type}));});images.set(image.cid.replace(/^<|>$/g,''),data);
    }catch{/* A missing image never prevents reading the draft text. */}
   }
  }));
  if(!current()||abort.signal.aborted)return '';
  for(const element of doc.querySelectorAll('[src],[srcset],[background]')){
   const src=element.getAttribute('src')??'';let cid='';try{if(/^cid:/i.test(src))cid=decodeURIComponent(src.slice(4)).replace(/^<|>$/g,'');}catch{/* Invalid CIDs remain blocked. */}
   const data=images.get(cid)??(/^data:image\/(png|jpe?g|gif|webp|avif|bmp);base64,/i.test(src)?src:undefined);
   if(element.tagName==='IMG'&&data)element.setAttribute('src',data);else element.removeAttribute('src');element.removeAttribute('srcset');element.removeAttribute('background');
  }
  return doc.body.innerHTML;
 });
 const appearance=():ReaderAppearance=>{const values=settings(),width=values['reader.contentWidth']?.value;return {colorMode:resolveReaderColorMode(isReaderColorChoice(values['reader.colorMode']?.value)?values['reader.colorMode']!.value as 'auto'|'original'|'adaptive'|'invert':'auto',app.appearance()),fontSize:Number(values['reader.fontSize']?.value??14),lineHeight:Number(values['reader.lineHeight']?.value??1.6),contentWidth:width==='comfortable'||width==='wide'?width:'full'};};
 const srcdoc=()=>bodyFrameDoc(mode(),{html:html()??'',text:props.draft.bodyText},{appearance:appearance(),themeVars:themeCssVars(app.theme(),{density:app.density()})});
 const from=()=>props.draft.fromOverride??app.identities().find(row=>row.id===props.draft.identityId)?.email??app.me()?.username??'';
 return <section class={`reader ${css.root}`} data-testid="draft-preview">
  <header class="reader__header"><div class={css.title}><h2 class="reader__subject">{props.draft.subject||t('mail-no-subject')}</h2><span class={css.badge}>{t('mail-draft-notice')}</span></div>
   <div class="reader__meta"><div class="reader__address-line"><span class="reader__address-label">{t('mail-compose-from')}</span><span>{from()}</span></div><For each={['to','cc','bcc'] as const}>{field=><Show when={props.draft[field]}><div class="reader__address-line"><span class="reader__address-label">{t(`mail-compose-${field}`)}</span><span>{props.draft[field]}</span></div></Show>}</For></div>
   <p class={css.saved}>{formatDateTime(props.draft.savedAt)}</p>
   <div class={css.actions}><button type="button" class="btn btn--ghost" onClick={props.onRefresh} title={t('common-refresh')}><Icon name="refresh"/>{t('common-refresh')}</button><button type="button" class="btn" data-testid="draft-preview-edit" onClick={props.onEdit}><Icon name="compose"/>{t('mail-draft-edit')}</button><button type="button" class="btn btn--ghost" onClick={props.onClose}><Icon name="close"/>{t('common-close')}</button></div>
  </header>
  <Show when={(props.draft.attachments?.length??0)>0}><details class={css.attachments}><summary>{t('mail-attachments')} ({props.draft.attachments!.length})</summary><For each={props.draft.attachments}>{file=><div class={css.file}><Icon name="attachment"/><span>{file.name}</span><span>{formatBytes(file.size)}</span><Show when={app.downloadUrl()&&mailFeature('allowDownload')&&allowsOriginalBytes(mode())&&decision(file)?.allowed&&!decision(file)?.warning}><a class="btn btn--ghost" href={attachmentPolicyUrl(buildDownloadUrl(app.downloadUrl()!,{accountId:props.draft.accountId,blobId:file.blobId,name:file.name,mime:file.contentType??'application/octet-stream'}),undefined,false,false,true)} download={file.name.replace(/[\\/]/g,'_')} onClick={event=>{if(!current())event.preventDefault();}} aria-label={`${t('common-message-preview-download')}: ${file.name}`}><Icon name="download"/></a></Show></div>}</For></details></Show>
  <Show when={!html.loading} fallback={<MessageLoading variant="skeleton" delayMs={120}/>}><ReaderZoomBox><iframe class="reader__frame" title={t('mail-message-body')} sandbox="" referrerPolicy="no-referrer" srcdoc={srcdoc()}/></ReaderZoomBox></Show>
 </section>;
}
