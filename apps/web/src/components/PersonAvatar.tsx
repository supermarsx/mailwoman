import {createEffect,createSignal,onCleanup,onMount,For,Show,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {apiFetch} from '../api/http.ts';
import {withBase} from '../api/basePath.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {loadCatalog,t} from '../i18n';
import {AVATAR_COLORS} from './senderIcons/palette.ts';
import {senderDomainIcon} from './senderIcons/match.ts';
import './person-avatar.css';
import './senderIcons/palette.css';

export function avatarPresentation(name:string,address:string):{initials:string;color:string;palette:number} {
 const label=(name.trim()||address.split('@')[0]||'?').replace(/[\p{Cc}\p{Cf}]/gu,'');
 const words=label.split(/[\s._-]+/u).filter(Boolean);const initials=words.slice(0,2).map(word=>Array.from(word)[0]??'').join('').toLocaleUpperCase().slice(0,4)||'?';
 let hash=2166136261;for(const character of (address.trim()||label).toLowerCase())hash=Math.imul(hash^character.codePointAt(0)!,16777619)>>>0;
 const palette=hash%AVATAR_COLORS.length;
 return {initials,color:AVATAR_COLORS[palette]!,palette};
}
/** Generated locally by default. Contact mode asks only our authenticated server;
 * it never requests a sender-provided URL or an external avatar service. */
export function PersonAvatar(props:{name?:string|undefined;address?:string|undefined}):JSX.Element {
 onMount(()=>void loadCatalog('avatars'));
 const [currentOwner,setCurrentOwner]=createSignal(preferenceAccount());onCleanup(observePreferenceAccount(setCurrentOwner));
 const mode=()=>settings()['contacts.avatarMode']?.value??'initials';
 const appearance=()=>avatarPresentation(props.name??'',props.address??'');const [photo,setPhoto]=createSignal('');
 const brand=()=>settings()['contacts.senderIcons']?.value===false?undefined:senderDomainIcon(props.address??'');
 createEffect(()=>{
  const address=props.address;const owner=currentOwner();const selected=mode();setPhoto('');
  if(selected!=='contacts'||!address||!owner)return;
  const controller=new AbortController();let url='';let active=true;
  onCleanup(()=>{active=false;controller.abort();if(url)URL.revokeObjectURL(url);});
  void (async()=>{
   try {
    const response=await apiFetch(withBase('/api/account/contact-photo'),{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({address}),signal:controller.signal});
    if(!response.ok||response.status===204||response.headers.get('content-type')!=='image/png')return;
    const reader=response.body?.getReader();if(!reader)return;const chunks:Uint8Array<ArrayBuffer>[]=[];let size=0;
    for(;;){const result=await reader.read();if(result.done)break;size+=result.value.byteLength;if(size>1024*1024){await reader.cancel();return;}chunks.push(new Uint8Array(result.value));}
    if(!active||!preferenceAccountIsCurrent(owner))return;
    url=URL.createObjectURL(new Blob(chunks,{type:'image/png'}));setPhoto(url);
   }catch{/* Unavailable photos fall back to local initials without retries. */}
  })();
 });
 return <Show when={mode()!=='off'}><span class="person-avatar" aria-hidden="true" data-avatar-color={appearance().palette} data-sender-icon={!photo()?brand()?.id:undefined} data-tooltip={!photo()&&brand()?t('avatars-domain-icon',{name:brand()!.name}):undefined}><Show when={photo()} fallback={<Show when={brand()} fallback={appearance().initials}>{icon=><svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" aria-hidden="true"><For each={icon().paths}>{path=><path d={path}/>}</For></svg>}</Show>}>{url=><img src={url()} alt="" onError={()=>setPhoto('')}/>}</Show></span></Show>;
}
