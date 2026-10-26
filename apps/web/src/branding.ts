import {createSignal,onMount} from 'solid-js';
import {publicConfig,type PublicConfig} from './api/publicConfig.ts';
import {transportBase} from './api/transport.ts';
const [branding,setBranding]=createSignal<PublicConfig>();
let pending:Promise<void>|undefined;
const originalIcons=new Map<HTMLLinkElement,{href:string;type:string}>();
/** Accept only server-owned asset paths, never an arbitrary configured URL. */
export function brandAssetUrl(value:string|null|undefined):string|undefined{return value&&/^\/api\/assets\/[a-f0-9]{64}$/.test(value)?`${transportBase()}${value}`:undefined;}
export function applyBranding(value:PublicConfig):void{
 document.title=value.brandName||'Mailwoman';
 setBranding(value);
 const icon=brandAssetUrl(value.faviconUrl);
 document.querySelectorAll<HTMLLinkElement>('link[rel="icon"],link[rel="shortcut icon"],link[rel="apple-touch-icon"]').forEach(link=>{
  if(!originalIcons.has(link))originalIcons.set(link,{href:link.getAttribute('href')??'',type:link.type});
  const old=originalIcons.get(link)!;link.setAttribute('href',icon??old.href);link.type=icon?'image/png':old.type;
 });
}
export async function refreshBranding():Promise<void>{if(pending)return pending;pending=publicConfig().then(applyBranding).catch(()=>{}).finally(()=>{pending=undefined;});return pending;}
export function useBranding(){onMount(()=>{void refreshBranding();});return branding;}
