import {vars} from './contract.css.ts';

const CHANNEL = 'mw-public-policy-theme';
const tokens = {bg:vars.color.bg,surface:vars.color.surface,text:vars.color.text,dim:vars.color.textDim,border:vars.color.borderStrong,accent:vars.color.accent,link:vars.color.link,radius:vars.radius.lg,font:vars.font.ui};
export function publicPolicyPalette():Record<string,string>{
 const css=getComputedStyle(document.documentElement);
 return {...Object.fromEntries(Object.entries(tokens).map(([key,value])=>[key,css.getPropertyValue(value.slice(4,-1)).trim()]).filter(([,value])=>!!value)),scheme:document.documentElement.getAttribute('data-appearance')??''};
}
/** A fragment carries appearance only, never credentials or account details. */
export function themedPolicyUrl(value:string):string{
 try{const url=new URL(value,location.href);if(url.origin!==location.origin||!/(?:^|\/)(?:privacy|cookies-policy)$/.test(url.pathname)||url.hash)return value;
 url.hash=new URLSearchParams({'mw-theme':JSON.stringify(publicPolicyPalette())}).toString();return url.href;
 }catch{return value;}
}
/** Same-origin policy tabs follow live theme changes without browser storage. */
export function installPublicPolicyThemeBridge():()=>void{
 if(typeof BroadcastChannel==='undefined')return ()=>undefined;
 let channel:BroadcastChannel;try{channel=new BroadcastChannel(CHANNEL);}catch{return ()=>undefined;}
 let active=true;
 const publish=()=>{if(active)channel.postMessage({type:'policy-theme',palette:publicPolicyPalette()});};
 channel.onmessage=event=>{if(event.data?.type==='request-theme')publish();};
 const observer=new MutationObserver(()=>queueMicrotask(publish));
 observer.observe(document.documentElement,{attributes:true,attributeFilter:['class','style','data-theme','data-appearance','data-accent','data-density']});
 return ()=>{active=false;observer.disconnect();channel.close();};
}
