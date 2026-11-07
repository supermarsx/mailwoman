import {createEffect,createMemo,createResource,createSignal,onCleanup,onMount,For,Show,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {apiFetch} from '../api/http.ts';
import {withBase} from '../api/basePath.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {loadCatalog,t} from '../i18n';
import {avatarPresentation} from './avatarPresentation.ts';
import {senderDomainIcon} from './senderIcons/match.ts';
import {brandAvatarDecision,DEFAULT_REQUIRE_AUTH,isRequireAuth,senderIconDecision} from '../brands/senderBrand.ts';
import {resourceValue} from './resourceValue.ts';
import {loadBrandPath,type BrandEntry} from '../brands/registry.ts';
import type {ListAuthSummary} from '../api/jmap-types.ts';
import './person-avatar.css';
import './senderIcons/palette.css';
import './senderIcons/brands.css';

export {avatarPresentation};
/** A registry brand's mark. The avatar box is fixed-size, so the initials stay until the lazily loaded path arrives and nothing shifts. */
function RegistryMark(props:{brand:BrandEntry;fallback:string}):JSX.Element {
 const [path]=createResource(()=>props.brand.id,loadBrandPath);
 return <Show when={resourceValue(path)} fallback={props.fallback}>{d=><svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true"><path d={d()} fill={props.brand.color??'currentColor'}/></svg>}</Show>;
}
/** Generated locally by default. Contact mode asks only our authenticated server;
 * it never requests a sender-provided URL or an external avatar service. */
export function PersonAvatar(props:{name?:string|undefined;address?:string|undefined;
 /** A message's sender: a bundled brand icon is then shown only when `auth` proves the message (DMARC aligned, DKIM for that brand), else the initials. Contacts and other non-message uses leave it off. */
 messageSender?:boolean|undefined;auth?:ListAuthSummary|null|undefined}):JSX.Element {
 onMount(()=>void loadCatalog('avatars'));
 const [currentOwner,setCurrentOwner]=createSignal(preferenceAccount());onCleanup(observePreferenceAccount(setCurrentOwner));
 const mode=()=>settings()['contacts.avatarMode']?.value??'initials';
 const appearance=()=>avatarPresentation(props.name??'',props.address??'');const [photo,setPhoto]=createSignal('');
 const requireAuth=()=>{const value=settings()['mail.brandAvatars.requireAuth']?.value;return isRequireAuth(value)?value:DEFAULT_REQUIRE_AUTH;};
 /** One decision per row: a catalogue icon, else a registry brand (rows only), else initials (with `unverified` set when authentication took the icon away). */
 const shown=createMemo(()=>{
  if(settings()['contacts.senderIcons']?.value===false)return {};
  const address=props.address??'';const icon=senderDomainIcon(address);
  if(icon){if(!props.messageSender)return {icon};return senderIconDecision(address,icon,props.auth,senderDomainIcon,requireAuth())==='show'?{icon}:{unverified:true};}
  if(!props.messageSender||settings()['mail.brandAvatars']?.value===false)return {};
  const decision=brandAvatarDecision(address,props.auth,requireAuth());return decision.brand?{mark:decision.brand}:{unverified:decision.unverified};
 });
 const brand=()=>shown().icon;
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
 return <Show when={mode()!=='off'}><span class="person-avatar" aria-hidden="true" data-avatar-color={appearance().palette} data-sender-icon={!photo()?(brand()?.id??shown().mark?.id):undefined} data-unverified={!photo()&&shown().unverified?'true':undefined} data-tooltip={!photo()?(brand()?t('avatars-domain-icon',{name:brand()!.name}):shown().mark?t('avatars-domain-icon',{name:shown().mark!.name}):shown().unverified?t('avatars-unverified-sender'):undefined):undefined}><Show when={photo()} fallback={<Show when={brand()} fallback={<Show when={shown().mark} fallback={appearance().initials}>{mark=><RegistryMark brand={mark()} fallback={appearance().initials}/>}</Show>}>{icon=><Show when={icon().asset} fallback={<Show when={icon().monogram} fallback={<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" aria-hidden="true"><For each={icon().paths}>{path=><path d={path}/>}</For></svg>}>{text=><span class="person-avatar__monogram" data-length={text().length}>{text()}</span>}</Show>}>{asset=><img class="person-avatar__brand" src={asset()} alt=""/>}</Show>}</Show>}>{url=><img src={url()} alt="" onError={()=>setPhoto('')}/>}</Show></span></Show>;
}
