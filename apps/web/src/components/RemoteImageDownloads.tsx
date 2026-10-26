import {createSignal,createEffect,on,For,onCleanup,Show,type JSX} from 'solid-js';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {apiFetch} from '../api/http.ts';
import {signedSource,signContent} from '../api/remote-content.ts';
import {createObjectUrlOwner} from '../viewers/objectUrl.ts';
import {t} from '../i18n';
import {toast} from './toast/toast.ts';
import {Icon} from './Icon.tsx';

export function RemoteImageDownloads(props:{emailId:string;scope:string;urls:readonly string[]}):JSX.Element {
 const [pending,setPending]=createSignal(false),[failed,setFailed]=createSignal(false),objects=createObjectUrlOwner();
 let abort:AbortController|undefined;let disposed=false;let generation=0;
 onCleanup(()=>{disposed=true;abort?.abort();});
 createEffect(on(()=>props.scope,(scope,previous)=>{if(scope===previous)return;generation++;abort?.abort();setPending(false);setFailed(false);}));
 async function download(url:string):Promise<void>{
  const owner=preferenceAccount(),scope=props.scope;if(!owner||pending()||!props.urls.includes(url))return;
  const requestGeneration=++generation;
  const current=()=>!disposed&&requestGeneration===generation&&!controller.signal.aborted&&scope===props.scope&&preferenceAccountIsCurrent(owner);
  abort=new AbortController();const controller=abort;
  const unwatch=observePreferenceAccount(()=>{if(!current()){controller.abort();if(requestGeneration===generation)setPending(false);}});
  setPending(true);setFailed(false);
  try{
   const signed=await signContent(props.emailId,{imageDownloads:[url]},owner.id);
   if(!current())return;
   const path=signed.imageDownloads?.find(item=>item.url===url)?.src,src=path?signedSource(path,'d'):null;
   if(!src)throw new Error('Image download refused');
   const response=await apiFetch(src,{signal:controller.signal,headers:{'X-Mailwoman-Account':owner.id}});
   if(!response.ok||!response.headers.get('content-type')?.startsWith('image/'))throw new Error('Image download failed');
   const blob=await response.blob();if(!current())return;
   const object=objects.adopt(URL.createObjectURL(blob));const link=document.createElement('a');link.href=object;link.download='message-image.png';link.rel='noopener';document.body.append(link);link.click();link.remove();
   setTimeout(()=>objects.release(object),30000);toast.success(t('privacy-image-download-ready'));
  }catch{if(current()){setFailed(true);toast.error(t('privacy-image-download-failed'));}}
  finally{unwatch();if(current())setPending(false);}
 }
 return <Show when={props.urls.length>0||pending()||failed()}><section aria-label={t('privacy-image-downloads')}><details><summary>{t('privacy-image-downloads')}</summary><p>{t('privacy-image-download-help')}</p><For each={props.urls}>{url=><p><span dir="auto">{new URL(url).hostname}{new URL(url).pathname}</span> <button type="button" class="btn btn--ghost" disabled={pending()} onClick={()=>void download(url)}><Icon name="download"/>{t('privacy-download-file')}</button></p>}</For></details><Show when={failed()}><p role="alert">{t('privacy-image-download-failed')}</p></Show></section></Show>;
}
